pub mod builtin_function;
pub mod legalize_decl;
pub mod legalize_expr;
pub mod legalize_init;
pub mod legalize_stmt;
pub mod legalize_type;
#[cfg(test)]
pub mod tests;

use crate::{
    ast::{
        AttributeKind, Designation, DesignationKind, Initializer, InitializerKind, TranslationUnit,
        decl::DeclarationKind,
        expr::{BinOpKind, CastMethod, Expr, ExprKind, UnaryOpKind},
    },
    ctype::{Type, TypeKind, cast::wrap_implicit_cast, complex_part_type, get_inner_type},
    symtab::SymbolTable,
    variant::Variant,
};
use codespan::Span;
use codespan_reporting::diagnostic::Diagnostic;
use num::ToPrimitive;
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

pub struct Legalizer {
    pub symtabs: Vec<Rc<RefCell<SymbolTable>>>,
    pub xlen: usize,
    pub builtin_functions: BTreeMap<String, Rc<RefCell<TranslationUnit>>>,
}

impl Legalizer {
    pub fn new() -> Legalizer {
        Legalizer {
            symtabs: vec![],
            xlen: 64,
            builtin_functions: BTreeMap::new(),
        }
    }

    pub fn legalize(
        &mut self,
        ast: &Rc<RefCell<TranslationUnit>>,
    ) -> Result<(), Diagnostic<usize>> {
        if let Some(symtab) = &ast.borrow().symtab {
            self.enter_scope(Rc::clone(symtab));
        }
        for decl in &ast.borrow_mut().decls {
            self.visit_declaration(decl)?;
        }

        //tu和ast应该是共用一个符号表的
        for (_, tu) in &self.builtin_functions {
            for decl in &tu.borrow().decls {
                match &decl.borrow().kind {
                    DeclarationKind::Function { body: None, .. } => {
                        ast.borrow_mut().decls.insert(0, decl.clone())
                    }
                    _ => ast.borrow_mut().decls.push(decl.clone()),
                }
            }
        }

        if let Some(_) = ast.borrow().symtab {
            self.leave_scope();
        }
        Ok(())
    }

    pub fn enter_scope(&mut self, symtab: Rc<RefCell<SymbolTable>>) {
        self.symtabs.push(symtab);
    }

    pub fn leave_scope(&mut self) {
        self.symtabs.pop();
    }

    //得到类型在被合法化之前的复数实部或者虚部的类型
    pub fn get_complex_part_type(&self, r#type: &Rc<RefCell<Type>>) -> Option<Rc<RefCell<Type>>> {
        let r#type = get_inner_type(r#type.clone());

        for attribute in &r#type.borrow().attributes {
            if let AttributeKind::TypeBeforeLegalize { origin_type } = &attribute.borrow().kind {
                return complex_part_type(origin_type);
            }
        }

        None
    }

    pub fn was_complex(&self, r#type: &Rc<RefCell<Type>>) -> bool {
        if let Some(_) = self.get_complex_part_type(r#type) {
            true
        } else {
            false
        }
    }

    //返回值中的大小以bit为单位
    pub fn get_bitint_info(&self, r#type: &Rc<RefCell<Type>>) -> Option<(bool, usize)> {
        let r#type = get_inner_type(r#type.clone());

        for attribute in &r#type.borrow().attributes {
            if let AttributeKind::TypeBeforeLegalize { origin_type } = &attribute.borrow().kind {
                //整数类型被合法化只有bitint一种情况
                match &get_inner_type(origin_type.clone()).borrow().kind {
                    TypeKind::BitInt {
                        unsigned,
                        width_expr,
                    } => match &width_expr.borrow().value {
                        //直接用 .size 得到的是四舍五入后的以byte为单位的大小
                        Variant::Int(a) => return Some((*unsigned, a.to_usize().unwrap_or(0))),
                        _ => {}
                    },
                    t if t.is_integer() => match t.size() {
                        None | Some(1) | Some(2) | Some(4) => {}
                        Some(8) if self.xlen == 64 => {}
                        Some(size) => return Some((t.is_unsigned().unwrap_or(false), size * 8)),
                    },
                    _ => {}
                }
            }
        }

        None
    }

    pub fn was_bitint(&self, r#type: &Rc<RefCell<Type>>) -> bool {
        if let Some(_) = self.get_bitint_info(r#type) {
            true
        } else {
            false
        }
    }

    pub fn extract_member(
        &self,
        node: &Rc<RefCell<Expr>>,
        part: &str,
    ) -> Option<Rc<RefCell<Expr>>> {
        let Expr {
            file_id,
            span,
            kind,
            r#type,
            ..
        } = &*node.borrow();

        match kind {
            ExprKind::CompoundLiteral { initializer, .. } => match &initializer.borrow().kind {
                InitializerKind::Braced(initializers) => {
                    for initializer in initializers {
                        let Initializer {
                            designations, kind, ..
                        } = &*initializer.borrow();
                        for designation in designations {
                            if let DesignationKind::MemberAccess(name) = &designation.kind
                                && name == part
                            {
                                let InitializerKind::Expr(expr) = kind else {
                                    continue;
                                };
                                return Some(expr.clone());
                            }
                        }
                    }
                    None
                }
                _ => None,
            },
            ExprKind::Cast {
                target,
                method: CastMethod::LToRValue,
                ..
            } => {
                return self.extract_member(target, part);
            }
            _ if self.was_complex(r#type) || self.was_bitint(r#type) => {
                let TypeKind::Record {
                    members: Some(members),
                    ..
                } = &r#type.borrow().kind
                else {
                    unreachable!()
                };
                let r#type = members.get(part).unwrap().borrow().r#type.clone();

                return Some(Rc::new(RefCell::new(wrap_implicit_cast(
                    Rc::new(RefCell::new(Expr {
                        r#type: r#type.clone(),
                        is_lvalue: true,
                        symbol: Some(members.get(part).unwrap().clone()),
                        has_side_effects: false,
                        ..Expr::new(
                            *file_id,
                            *span,
                            ExprKind::MemberAccess {
                                target: node.clone(),
                                is_arrow: false,
                                name: part.to_string(),
                            },
                        )
                    })),
                    r#type.clone(),
                    CastMethod::LToRValue,
                ))));
            }
            _ if part == "real" || part == "w0" => Some(node.clone()),
            _ => None,
        }
    }

    pub fn make_unary_op(
        &self,
        file_id: usize,
        span: Span,
        op: UnaryOpKind,
        operand: &Rc<RefCell<Expr>>,
        r#type: Option<Rc<RefCell<Type>>>,
    ) -> Rc<RefCell<Expr>> {
        Rc::new(RefCell::new(Expr {
            r#type: r#type.unwrap_or(operand.borrow().r#type.clone()),
            has_side_effects: false,
            ..Expr::new(
                file_id,
                span,
                ExprKind::UnaryOp {
                    op,
                    operand: operand.clone(),
                },
            )
        }))
    }

    pub fn make_binary_op(
        &self,
        file_id: usize,
        span: Span,
        op: BinOpKind,
        left: Option<Rc<RefCell<Expr>>>,
        right: Option<Rc<RefCell<Expr>>>,
        r#type: Option<Rc<RefCell<Type>>>,
    ) -> Rc<RefCell<Expr>> {
        if let Some(left) = &left
            && let Some(right) = &right
        {
            Rc::new(RefCell::new(Expr {
                r#type: r#type.unwrap_or(left.borrow().r#type.clone()),
                has_side_effects: false,
                ..Expr::new(
                    file_id,
                    span,
                    ExprKind::BinOp {
                        op,
                        left: left.clone(),
                        right: right.clone(),
                    },
                )
            }))
        } else if let Some(left) = &left {
            left.clone()
        } else if let Some(right) = &right {
            right.clone()
        } else {
            unreachable!()
        }
    }

    pub fn make_compound(
        &self,
        file_id: usize,
        span: Span,
        members: Vec<(String, Rc<RefCell<Expr>>)>,
        r#type: Rc<RefCell<Type>>,
    ) -> Rc<RefCell<Expr>> {
        let mut initializers = vec![];

        for (name, expr) in members {
            initializers.push(Rc::new(RefCell::new(Initializer {
                designations: vec![Designation {
                    file_id,
                    span,
                    kind: DesignationKind::MemberAccess(name),
                }],
                r#type: expr.borrow().r#type.clone(),
                has_side_effects: false,
                ..Initializer::new(file_id, span, InitializerKind::Expr(expr.clone()))
            })));
        }

        Rc::new(RefCell::new(wrap_implicit_cast(
            Rc::new(RefCell::new(Expr {
                r#type: r#type.clone(),
                is_lvalue: true,
                ..Expr::new(
                    file_id,
                    span,
                    ExprKind::CompoundLiteral {
                        decls: vec![],
                        storage_classes: vec![],
                        initializer: Rc::new(RefCell::new(Initializer {
                            r#type: r#type.clone(),
                            has_side_effects: false,
                            ..Initializer::new(file_id, span, InitializerKind::Braced(initializers))
                        })),
                    },
                )
            })),
            r#type.clone(),
            CastMethod::LToRValue,
        )))
    }

    pub fn make_builtin_call(
        &self,
        file_id: usize,
        span: Span,
        name: &str,
        arguments: Vec<Rc<RefCell<Expr>>>,
        return_type: &Rc<RefCell<Type>>,
    ) -> Rc<RefCell<Expr>> {
        Rc::new(RefCell::new(Expr {
            r#type: return_type.clone(),
            ..Expr::new(
                file_id,
                span,
                ExprKind::FunctionCall {
                    target: Rc::new(RefCell::new(Expr {
                        r#type: Rc::new(RefCell::new(Type {
                            kind: TypeKind::Pointer(Rc::new(RefCell::new(Type {
                                //不区分具体的函数类型
                                kind: TypeKind::Void,
                                ..Type::new(file_id, span)
                            }))),
                            ..Type::new(file_id, span)
                        })),
                        has_side_effects: false,
                        ..Expr::new(file_id, span, ExprKind::Name(name.to_string()))
                    })),
                    arguments,
                },
            )
        }))
    }
}
