use super::TypeChecker;
use crate::{
    ast::decl::{Declaration, DeclarationKind, StorageClassKind},
    ctype::{Type, TypeKind, as_parameter_type},
    diagnostic::{Error, ErrorKind},
    symtab::{Namespace, Symbol, SymbolKind, SymbolTable},
    typechecker::Context,
    variant::Variant,
};
use indexmap::IndexMap;
use num::{BigInt, ToPrimitive};
use std::{cell::RefCell, rc::Rc};

impl<'a> TypeChecker<'a> {
    pub fn reassign(&mut self, r#type: &mut Rc<RefCell<Type<'a>>>) -> Result<(), Error<'a>> {
        let mut new_type = None;
        {
            let mut r#type = r#type.borrow_mut();
            match &mut r#type.kind {
                TypeKind::Record { name, .. } | TypeKind::Enum { name, .. } => {
                    if let Some(t) = &self.cur_symtab.borrow().lookup(Namespace::Tag, name) {
                        new_type = Some(Rc::clone(&t.borrow().r#type));
                    }
                }
                TypeKind::Typedef { name, .. } => {
                    if let Some(t) = &self.cur_symtab.borrow().lookup(Namespace::Ordinary, name) {
                        new_type = Some(Rc::clone(&t.borrow().r#type));
                    }
                }
                TypeKind::Function {
                    return_type,
                    parameters_type,
                    has_varparam,
                } => {
                    new_type = Some(Rc::new(RefCell::new(Type {
                        kind: TypeKind::Function {
                            return_type: Rc::clone(return_type),
                            parameters_type: {
                                let mut t = Vec::new();
                                for p in parameters_type {
                                    t.push(as_parameter_type(Rc::clone(p)));
                                }
                                t
                            },
                            has_varparam: *has_varparam,
                        },
                        span: r#type.span,
                        attributes: r#type.attributes.clone(),
                    })))
                }
                TypeKind::Array {
                    element_type,
                    len_expr,
                    ..
                } => {
                    self.reassign(element_type)?;
                    if let Some(len_expr) = len_expr {
                        self.visit_expr(Rc::clone(&len_expr))?;
                    }
                }
                TypeKind::Atomic(r#type) => self.reassign(r#type)?,
                TypeKind::Auto(r#type) => {
                    if let Some(t) = r#type {
                        self.reassign(t)?;
                    }
                }
                TypeKind::Pointer(r#type) => self.reassign(r#type)?,
                TypeKind::Qualified { r#type, .. } => self.reassign(r#type)?,
                TypeKind::Typeof { r#type, expr, .. } => {
                    if let Some(t) = r#type {
                        self.reassign(t)?;
                    }
                    if let Some(expr) = expr {
                        self.contexts.push(Context::Typeof);
                        self.visit_expr(Rc::clone(expr))?;
                        self.contexts.pop();
                    }
                }
                _ => {}
            }
        }

        if let Some(new_type) = new_type {
            *r#type = new_type;
        }

        Ok(())
    }

    pub fn visit_declaration(
        &mut self,
        node: Rc<RefCell<Declaration<'a>>>,
    ) -> Result<(), Error<'a>> {
        self.contexts
            .push(Context::Decl(node.borrow().kind.clone()));

        {
            let mut node = node.borrow_mut();
            self.reassign(&mut node.r#type)?;

            match &node.kind {
                DeclarationKind::Parameter => {
                    node.r#type = as_parameter_type(Rc::clone(&node.r#type));
                }
                _ => {}
            }
        }

        let node = node.borrow();

        if node.storage_classes.len() > 1 {
            return Err(Error {
                span: node.storage_classes[1].span,
                kind: ErrorKind::TooManyStorageClass,
            });
        }
        match &node.kind {
            DeclarationKind::Var { initializer } => {
                self.cur_symtab.borrow_mut().add(
                    Namespace::Ordinary,
                    Rc::new(RefCell::new(Symbol {
                        define_span: match initializer {
                            Some(_) => Some(node.span),
                            None => None,
                        },
                        declare_spans: vec![node.span],
                        name: node.name.clone(),
                        kind: SymbolKind::Object {
                            storage_classes: node.storage_classes.clone(),
                        },
                        r#type: Rc::clone(&node.r#type),
                        attributes: node.attributes.clone(),
                    })),
                )?;
                if let Some(initializer) = initializer {
                    self.visit_initializer(Rc::clone(initializer), Rc::clone(&node.r#type))?;
                }
            }
            DeclarationKind::Function {
                parameter_decls,
                function_specs,
                body,
            } => {
                let parent_symtab = Rc::clone(&self.cur_symtab);

                if let Some(_) = body {
                    self.enter_scope();
                    self.func_symtab.push(Rc::clone(&self.cur_symtab));
                } else {
                    //函数声明需要解析参数声明, 但只做类型检查, 添加的符号将被忽略
                    self.cur_symtab = Rc::new(RefCell::new(SymbolTable::new()));
                }

                for decl in parameter_decls {
                    self.visit_declaration(Rc::clone(decl))?;
                }

                parent_symtab.borrow_mut().add(
                    Namespace::Ordinary,
                    Rc::new(RefCell::new(Symbol {
                        define_span: match body {
                            Some(_) => Some(node.span),
                            None => None,
                        },
                        declare_spans: vec![node.span],
                        name: node.name.clone(),
                        kind: SymbolKind::Function {
                            function_specs: function_specs.to_vec(),
                        },
                        r#type: Rc::clone(&node.r#type),
                        attributes: node.attributes.clone(),
                    })),
                )?;

                if let Some(body) = body {
                    self.visit_stmt(Rc::clone(body))?;

                    self.func_symtab.pop();
                    self.leave_scope();
                } else {
                    self.cur_symtab = parent_symtab;
                }
            }
            DeclarationKind::Parameter => {
                for storage_class in &node.storage_classes {
                    if storage_class.kind != StorageClassKind::Register {
                        return Err(Error {
                            span: storage_class.span,
                            kind: ErrorKind::UnexpectStorageClass,
                        });
                    }
                }
                self.cur_symtab.borrow_mut().add(
                    Namespace::Ordinary,
                    Rc::new(RefCell::new(Symbol {
                        define_span: Some(node.span),
                        declare_spans: vec![node.span],
                        name: node.name.clone(),
                        kind: SymbolKind::Parameter {
                            storage_classes: node.storage_classes.clone(),
                        },
                        r#type: Rc::clone(&node.r#type),
                        attributes: node.attributes.clone(),
                    })),
                )?;
            }
            DeclarationKind::Type => {
                self.cur_symtab.borrow_mut().add(
                    Namespace::Ordinary,
                    Rc::new(RefCell::new(Symbol {
                        define_span: Some(node.span),
                        declare_spans: vec![node.span],
                        name: node.name.clone(),
                        kind: SymbolKind::Type,
                        r#type: Rc::new(RefCell::new(Type {
                            span: node.span,
                            attributes: vec![],
                            kind: TypeKind::Typedef {
                                name: node.name.clone(),
                                r#type: Some(Rc::clone(&node.r#type)),
                            },
                        })),
                        attributes: node.attributes.clone(),
                    })),
                )?;
            }
            DeclarationKind::Record { members_decl } => {
                self.cur_symtab.borrow_mut().add(
                    Namespace::Tag,
                    Rc::new(RefCell::new(Symbol {
                        define_span: match members_decl {
                            Some(_) => Some(node.span),
                            None => None,
                        },
                        declare_spans: vec![node.span],
                        name: node.name.clone(),
                        r#type: Rc::clone(&node.r#type),
                        kind: SymbolKind::Record,
                        attributes: node.attributes.clone(),
                    })),
                )?;

                if let Some(members_decl) = members_decl {
                    self.records.push(Rc::clone(&node.r#type));
                    for decl in members_decl {
                        self.visit_declaration(Rc::clone(&decl))?;
                    }
                    self.records.pop();
                }
            }
            DeclarationKind::Member { bit_field } => {
                if node.storage_classes.len() > 0 {
                    return Err(Error {
                        span: node.storage_classes[0].span,
                        kind: ErrorKind::TooManyStorageClass,
                    });
                }
                if let Some(t) = bit_field {
                    self.visit_expr(Rc::clone(t))?;
                }
                let symbol = Rc::new(RefCell::new(Symbol {
                    define_span: Some(node.span),
                    declare_spans: vec![node.span],
                    name: node.name.clone(),
                    kind: SymbolKind::Member {
                        bit_field: match bit_field {
                            Some(t) => match &t.borrow().value {
                                Variant::Int(value) => match value.to_usize() {
                                    Some(t) => Some(t),
                                    None => {
                                        return Err(Error {
                                            span: t.borrow().span,
                                            kind: ErrorKind::Other(format!("invalid bit field")),
                                        });
                                    }
                                },
                                _ => {
                                    return Err(Error {
                                        span: t.borrow().span,
                                        kind: ErrorKind::Other(format!("invalid bit field")),
                                    });
                                }
                            },
                            None => None,
                        },
                    },
                    r#type: Rc::clone(&node.r#type),
                    attributes: node.attributes.clone(),
                }));
                match &mut self.records.last().unwrap().borrow_mut().kind {
                    TypeKind::Record { members, .. } => {
                        if let Some(t) = members {
                            if t.contains_key(&node.name) {
                                return Err(Error {
                                    span: node.span,
                                    kind: ErrorKind::Redefine(node.name.clone()),
                                });
                            }
                            t.insert(node.name.clone(), symbol);
                        } else {
                            let mut t = IndexMap::new();
                            t.insert(node.name.clone(), symbol);
                            *members = Some(t);
                        }
                    }
                    _ => unreachable!(),
                }
            }
            DeclarationKind::Enum { enumerators } => {
                self.cur_symtab.borrow_mut().add(
                    Namespace::Tag,
                    Rc::new(RefCell::new(Symbol {
                        define_span: match enumerators {
                            Some(_) => Some(node.span),
                            None => None,
                        },
                        declare_spans: vec![node.span],
                        name: node.name.clone(),
                        kind: SymbolKind::Enum,
                        r#type: Rc::clone(&node.r#type),
                        attributes: node.attributes.clone(),
                    })),
                )?;

                if let Some(enumerators) = enumerators {
                    self.enums.push(Rc::clone(&node.r#type));
                    for decl in enumerators {
                        self.visit_declaration(Rc::clone(&decl))?;
                    }
                    self.enums.pop();

                    let TypeKind::Enum {
                        underlying,
                        enum_consts: Some(enum_consts),
                        ..
                    } = &mut node.r#type.borrow_mut().kind
                    else {
                        return Err(Error {
                            span: node.span,
                            kind: ErrorKind::Other(format!("incomplete enum")),
                        });
                    };

                    let mut kind = None;
                    if let TypeKind::Error = underlying.borrow().kind {
                        //如果实际最小的大于0, 最大的小于0也不影响对底层类型的推断
                        let mut min = BigInt::ZERO;
                        let mut max = BigInt::ZERO;
                        for (_name, enum_const) in enum_consts {
                            match &enum_const.borrow().kind {
                                SymbolKind::EnumConst { value } => {
                                    max = value.max(&max).clone();
                                    min = value.min(&min).clone();
                                }
                                _ => {}
                            }
                        }

                        kind = Some(TypeKind::Int);
                        for candidiate in vec![
                            TypeKind::UInt,
                            TypeKind::Int,
                            TypeKind::UShort,
                            TypeKind::Short,
                            TypeKind::ULong,
                            TypeKind::Long,
                            TypeKind::ULongLong,
                            TypeKind::LongLong,
                        ] {
                            let Some((int_min, int_max)) = candidiate.range() else {
                                continue;
                            };
                            kind = Some(candidiate);
                            if int_min <= min && max <= int_max {
                                break;
                            }
                        }
                    }
                    if let Some(kind) = kind {
                        let r#type = Rc::new(RefCell::new(Type {
                            span: underlying.borrow().span,
                            attributes: underlying.borrow().attributes.clone(),
                            kind,
                        }));
                        *underlying = r#type;
                    }
                }
            }
            DeclarationKind::Enumerator { value } => {
                let mut const_value = match value {
                    Some(t) => match &t.borrow().value {
                        Variant::Int(value) => Some(value.clone()),
                        _ => None,
                    },
                    None => None,
                };

                let symbol = Rc::new(RefCell::new(Symbol {
                    define_span: Some(node.span),
                    declare_spans: vec![node.span],
                    name: node.name.clone(),
                    kind: SymbolKind::EnumConst {
                        value: const_value.clone().unwrap_or(BigInt::ZERO),
                    },
                    r#type: Rc::clone(self.enums.last().unwrap()),
                    attributes: node.attributes.clone(),
                }));

                self.cur_symtab
                    .borrow_mut()
                    .add(Namespace::Ordinary, Rc::clone(&symbol))?;

                if let Some(t) = value {
                    self.visit_expr(Rc::clone(t))?;
                }

                match &mut self.enums.last_mut().unwrap().borrow_mut().kind {
                    TypeKind::Enum { enum_consts, .. } => {
                        if let Some(t) = enum_consts {
                            //如果枚举常量名称重复的话, 在之前加入符号表时就会出错
                            t.insert(node.name.clone(), Rc::clone(&symbol));

                            if let None = const_value
                                && let Some(t) = t.get(&node.name)
                            {
                                match &t.borrow().kind {
                                    SymbolKind::EnumConst { value: pre_value } => {
                                        const_value = Some(pre_value + 1);
                                    }
                                    _ => {}
                                }
                            }
                        } else {
                            if let None = const_value {
                                const_value = Some(BigInt::ZERO);
                            }

                            let mut t = IndexMap::new();
                            t.insert(node.name.clone(), Rc::clone(&symbol));
                            *enum_consts = Some(t);
                        }
                    }
                    _ => unreachable!(),
                }

                let SymbolKind::EnumConst { value } = &mut symbol.borrow_mut().kind else {
                    unreachable!();
                };
                *value = const_value.unwrap_or(BigInt::ZERO);
            }
            DeclarationKind::StaticAssert { .. } => {} //TODO static_assert声明
            DeclarationKind::Attribute => {}           //TODO 属性声明
        }

        self.contexts.pop();
        Ok(())
    }
}
