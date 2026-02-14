use crate::typechecker::TypeChecker;
use crate::{
    ast::{
        AttributeKind,
        decl::{Declaration, DeclarationKind},
    },
    ctype::{Type, TypeKind, TypeQual, as_parameter_type, cast::remove_qualifier},
    symtab::{Namespace, SymbolKind},
    typechecker::Context,
    variant::Variant,
};
use codespan_reporting::diagnostic::{Diagnostic, Label};
use num::ToPrimitive;
use std::{cell::RefCell, rc::Rc};

impl TypeChecker {
    pub fn check_type(&mut self, r#type: &mut Rc<RefCell<Type>>) -> Result<(), Diagnostic<usize>> {
        let mut new_type = None;
        {
            let mut r#type = r#type.borrow_mut();
            match &mut r#type.kind {
                TypeKind::Record { name, kind, .. } => {
                    let name = name.clone();
                    let kind = kind.clone();
                    if let Some(symbol) = &self.cur_symtab.borrow().lookup(Namespace::Tag, &name) {
                        match &symbol.borrow().kind {
                            SymbolKind::Record { kind: symbol_kind } if kind == *symbol_kind => {
                                new_type = Some(Rc::clone(&symbol.borrow().r#type));
                            }
                            _ => {
                                let (file_id, span) = symbol
                                    .borrow()
                                    .define_loc
                                    .unwrap_or(symbol.borrow().declare_locs[0]);
                                return Err(Diagnostic::error()
                                    .with_message(format!("'{name}' is not a {kind}"))
                                    .with_label(
                                        Label::primary(r#type.file_id, r#type.span)
                                            .with_message("current type"),
                                    )
                                    .with_label(
                                        Label::secondary(file_id, span)
                                            .with_message("previous symbol"),
                                    ));
                            }
                        }
                    } else {
                        new_type = Some(Rc::new(RefCell::new(r#type.clone())));
                        self.visit_declaration(Rc::new(RefCell::new(Declaration {
                            name,
                            r#type: Rc::clone(new_type.as_ref().unwrap()),
                            ..Declaration::new(
                                r#type.file_id,
                                r#type.span,
                                DeclarationKind::Record { members_decl: None },
                            )
                        })))?;
                    }
                }
                TypeKind::Enum { name, .. } => {
                    let name = name.clone();
                    if let Some(symbol) = &self.cur_symtab.borrow().lookup(Namespace::Tag, &name) {
                        match &symbol.borrow().kind {
                            SymbolKind::Enum => {
                                new_type = Some(Rc::clone(&symbol.borrow().r#type));
                            }
                            _ => {
                                let (file_id, span) = symbol
                                    .borrow()
                                    .define_loc
                                    .unwrap_or(symbol.borrow().declare_locs[0]);
                                return Err(Diagnostic::error()
                                    .with_message(format!("'{name}' is not a enum"))
                                    .with_label(
                                        Label::primary(r#type.file_id, r#type.span)
                                            .with_message("current type"),
                                    )
                                    .with_label(
                                        Label::secondary(file_id, span)
                                            .with_message("previous symbol"),
                                    ));
                            }
                        }
                    } else {
                        new_type = Some(Rc::new(RefCell::new(r#type.clone())));
                        self.visit_declaration(Rc::new(RefCell::new(Declaration {
                            name,
                            r#type: Rc::clone(new_type.as_ref().unwrap()),
                            ..Declaration::new(
                                r#type.file_id,
                                r#type.span,
                                DeclarationKind::Enum { enumerators: None },
                            )
                        })))?;
                    }
                }
                TypeKind::Typedef { name, .. } => {
                    let name = name.clone();
                    let symbol = self
                        .cur_symtab
                        .borrow()
                        .lookup(Namespace::Ordinary, &name)
                        .ok_or(
                            Diagnostic::error()
                                .with_message(format!("undefined type '{name}'"))
                                .with_label(Label::primary(r#type.file_id, r#type.span)),
                        )?;
                    match symbol.borrow().kind {
                        SymbolKind::Type => new_type = Some(Rc::clone(&symbol.borrow().r#type)),
                        _ => {
                            let (file_id, span) = symbol
                                .borrow()
                                .define_loc
                                .unwrap_or(symbol.borrow().declare_locs[0]);
                            return Err(Diagnostic::error()
                                .with_message(format!("'{name}' is not a type"))
                                .with_label(
                                    Label::primary(r#type.file_id, r#type.span)
                                        .with_message("current type"),
                                )
                                .with_label(
                                    Label::secondary(file_id, span).with_message("previous symbol"),
                                ));
                        }
                    }
                }
                TypeKind::Function {
                    return_type,
                    parameters_type,
                    has_varparam,
                } => {
                    if return_type.borrow().is_array() {
                        return Err(Diagnostic::error()
                            .with_message(format!("function cannot return array type"))
                            .with_label(Label::primary(
                                return_type.borrow().file_id,
                                return_type.borrow().span,
                            )));
                    }
                    self.check_type(return_type)?;
                    new_type = Some(Rc::new(RefCell::new(Type {
                        kind: TypeKind::Function {
                            return_type: remove_qualifier(Rc::clone(return_type)),
                            parameters_type: {
                                let mut t = Vec::new();
                                for (i, p) in parameters_type.iter_mut().enumerate() {
                                    if p.borrow().is_void() && i > 0 {
                                        return Err(Diagnostic::error().with_message("void' must be the first and only parameter if specified").with_label(Label::primary(p.borrow().file_id, p.borrow().span)));
                                    }
                                    self.check_type(p)?;
                                    t.push(as_parameter_type(Rc::clone(p)));
                                }
                                t
                            },
                            has_varparam: *has_varparam,
                        },
                        attributes: r#type.attributes.clone(),
                        ..Type::new(r#type.file_id, r#type.span)
                    })))
                }
                TypeKind::Array {
                    element_type,
                    len_expr,
                    ..
                } => {
                    self.check_type(element_type)?;
                    if let Some(len_expr) = len_expr {
                        self.visit_expr(Rc::clone(&len_expr))?;
                        match &len_expr.borrow().value {
                            Variant::Int(value) => match value.to_u32() {
                                Some(_) => {}
                                None => {
                                    return Err(Diagnostic::error()
                                        .with_message(format!("invalid integer for array length"))
                                        .with_label(Label::primary(
                                            len_expr.borrow().file_id,
                                            len_expr.borrow().span,
                                        )));
                                }
                            },
                            //VLA的情况
                            Variant::Unknown => {}
                            _ => {
                                return Err(Diagnostic::error()
                                    .with_message(format!(
                                        "array length must be an integer constant"
                                    ))
                                    .with_label(Label::primary(
                                        len_expr.borrow().file_id,
                                        len_expr.borrow().span,
                                    )));
                            }
                        }
                    }
                }
                TypeKind::Atomic(r#type) => self.check_type(r#type)?,
                TypeKind::Auto(r#type) => {
                    if let Some(t) = r#type {
                        self.check_type(t)?;
                    }
                }
                TypeKind::Pointer(r#type) => self.check_type(r#type)?,
                TypeKind::Qualified { qualifiers, r#type } => {
                    self.check_type(r#type)?;
                    if qualifiers.contains(&TypeQual::Restrict) {
                        if !match &r#type.borrow().kind {
                            TypeKind::Pointer(t) => t.borrow().is_object(),
                            TypeKind::Array { .. } => true,
                            _ => false,
                        } {
                            return Err(Diagnostic::error().with_message( format!(
                                    "'restrict' requires an array or a pointer point to an object type"
                                )).with_label(Label::primary(r#type.borrow().file_id, r#type.borrow().span)));
                        }
                    }
                }
                TypeKind::Typeof { r#type, expr, .. } => {
                    if let Some(t) = r#type {
                        self.check_type(t)?;
                    }
                    if let Some(expr) = expr {
                        self.contexts.push(Context::Typeof);
                        self.visit_expr(Rc::clone(expr))?;
                        self.contexts.pop();
                    }
                }
                TypeKind::BitInt {
                    width_expr,
                    unsigned,
                    ..
                } => {
                    self.visit_expr(Rc::clone(width_expr))?;
                    match &width_expr.borrow().value {
                        Variant::Int(value) => match value.to_usize() {
                            Some(t) => {
                                if !*unsigned && t < 2 {
                                    return Err(Diagnostic::error()
                                        .with_message(
                                            "signed _BitInt must have a bit size of at least 2",
                                        )
                                        .with_label(Label::primary(
                                            width_expr.borrow().file_id,
                                            width_expr.borrow().span,
                                        )));
                                } else if t < 1 {
                                    return Err(Diagnostic::error()
                                        .with_message(
                                            "unsigned _BitInt must have a bit size of at least 1",
                                        )
                                        .with_label(Label::primary(
                                            width_expr.borrow().file_id,
                                            width_expr.borrow().span,
                                        )));
                                }
                            }
                            None => {
                                return Err(Diagnostic::error()
                                    .with_message(format!("invalid integer for _BitInt"))
                                    .with_label(Label::primary(
                                        width_expr.borrow().file_id,
                                        width_expr.borrow().span,
                                    )));
                            }
                        },
                        _ => {
                            return Err(Diagnostic::error()
                                .with_message(format!("_BitInt require an integer constant"))
                                .with_label(Label::primary(
                                    width_expr.borrow().file_id,
                                    width_expr.borrow().span,
                                )));
                        }
                    }
                }
                _ => {}
            }
        }

        if let Some(new_type) = new_type {
            *r#type = new_type;
        }

        for attribute in r#type.borrow_mut().attributes.iter_mut() {
            match &mut attribute.borrow_mut().kind {
                AttributeKind::AlignAs {
                    r#type: Some(r#type),
                    expr: None,
                } => self.check_type(r#type)?,
                AttributeKind::AlignAs {
                    r#type: None,
                    expr: Some(expr),
                } => {
                    self.visit_expr(Rc::clone(expr))?;
                    match &expr.borrow().value {
                        Variant::Int(value) => match value.to_usize() {
                            Some(_) => {}
                            None => {
                                return Err(Diagnostic::error()
                                    .with_message(format!("invalid integer for alignas"))
                                    .with_label(Label::primary(
                                        expr.borrow().file_id,
                                        expr.borrow().span,
                                    )));
                            }
                        },
                        _ => {
                            return Err(Diagnostic::error()
                                .with_message(format!(
                                    "alignas require an integer constant or a type"
                                ))
                                .with_label(Label::primary(
                                    expr.borrow().file_id,
                                    expr.borrow().span,
                                )));
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }
}
