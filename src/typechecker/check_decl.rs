use super::TypeChecker;
use crate::{
    ast::{
        AttributeKind,
        decl::{Declaration, DeclarationKind, StorageClassKind},
    },
    ctype::{Type, TypeKind, TypeQual, as_parameter_type, cast::remove_qualifier},
    diagnostic::{Diagnostic, DiagnosticKind},
    symtab::{Namespace, Symbol, SymbolKind, SymbolTable},
    typechecker::Context,
    variant::Variant,
};
use indexmap::IndexMap;
use num::BigInt;
use std::{cell::RefCell, rc::Rc};

impl<'a> TypeChecker<'a> {
    pub fn check_type(&mut self, r#type: &mut Rc<RefCell<Type<'a>>>) -> Result<(), Diagnostic<'a>> {
        let mut new_type = None;
        {
            let mut r#type = r#type.borrow_mut();
            match &mut r#type.kind {
                //如果声明节点的类型中含有record/enum, 那么在它之前一定有record/enum声明节点
                TypeKind::Record { name, kind, .. } => {
                    //这个type所属的node可能正好就是record声明, 所以如果找不到也不报错
                    let name = name.clone();
                    if let Some(t) = &self
                        .cur_symtab
                        .borrow()
                        .lookup_current(Namespace::Tag, &name)
                    {
                        match &t.borrow().kind {
                            SymbolKind::Record => {
                                match &t.borrow().r#type.borrow().kind {
                                    TypeKind::Record {
                                        kind: symbol_kind, ..
                                    } if *kind != *symbol_kind => {
                                        let kind = kind.clone();
                                        return Err(Diagnostic {
                                            span: r#type.span,
                                            kind: DiagnosticKind::Error,
                                            message: format!("'{name}' is not a '{kind}'"),
                                            notes: vec![Diagnostic {
                                                span: t
                                                    .borrow()
                                                    .define_span
                                                    .unwrap_or(t.borrow().declare_spans[0]),
                                                kind: DiagnosticKind::Note,
                                                message: format!("previous symbol"),
                                                notes: vec![],
                                            }],
                                        });
                                    }
                                    _ => {}
                                }
                                new_type = Some(Rc::clone(&t.borrow().r#type));
                            }
                            _ => {
                                return Err(Diagnostic {
                                    span: r#type.span,
                                    kind: DiagnosticKind::Error,
                                    message: format!("'{name}' is not a record"),
                                    notes: vec![Diagnostic {
                                        span: t
                                            .borrow()
                                            .define_span
                                            .unwrap_or(t.borrow().declare_spans[0]),
                                        kind: DiagnosticKind::Note,
                                        message: format!("previous symbol"),
                                        notes: vec![],
                                    }],
                                });
                            }
                        }
                    }
                }
                TypeKind::Enum { name, .. } => {
                    //这个type所属的node可能正好就是enum声明, 所以如果找不到也不报错
                    let name = name.clone();
                    if let Some(t) = &self
                        .cur_symtab
                        .borrow()
                        .lookup_current(Namespace::Tag, &name)
                    {
                        match t.borrow().kind {
                            SymbolKind::Enum => new_type = Some(Rc::clone(&t.borrow().r#type)),
                            _ => {
                                return Err(Diagnostic {
                                    span: r#type.span,
                                    kind: DiagnosticKind::Error,
                                    message: format!("'{name}' is not a enum"),
                                    notes: vec![Diagnostic {
                                        span: t
                                            .borrow()
                                            .define_span
                                            .unwrap_or(t.borrow().declare_spans[0]),
                                        kind: DiagnosticKind::Note,
                                        message: format!("previous symbol"),
                                        notes: vec![],
                                    }],
                                });
                            }
                        }
                    }
                }
                TypeKind::Typedef { name, .. } => {
                    let name = name.clone();
                    let symbol = self
                        .cur_symtab
                        .borrow()
                        .lookup(Namespace::Ordinary, &name)
                        .ok_or(Diagnostic {
                            span: r#type.span,
                            kind: DiagnosticKind::Error,
                            message: format!("undefined typedef type '{name}'"),
                            notes: vec![],
                        })?;
                    match symbol.borrow().kind {
                        SymbolKind::Type => new_type = Some(Rc::clone(&symbol.borrow().r#type)),
                        _ => {
                            return Err(Diagnostic {
                                span: r#type.span,
                                kind: DiagnosticKind::Error,
                                message: format!("'{name}' is not a type"),
                                notes: vec![Diagnostic {
                                    span: symbol
                                        .borrow()
                                        .define_span
                                        .unwrap_or(symbol.borrow().declare_spans[0]),
                                    kind: DiagnosticKind::Note,
                                    message: format!("previous symbol"),
                                    notes: vec![],
                                }],
                            });
                        }
                    }
                }
                TypeKind::Function {
                    return_type,
                    parameters_type,
                    has_varparam,
                } => {
                    if return_type.borrow().is_array() {
                        return Err(Diagnostic {
                            span: return_type.borrow().span,
                            kind: DiagnosticKind::Error,
                            message: format!("function cannot return array type"),
                            notes: vec![],
                        });
                    }
                    self.check_type(return_type)?;
                    new_type = Some(Rc::new(RefCell::new(Type {
                        kind: TypeKind::Function {
                            return_type: remove_qualifier(Rc::clone(return_type)),
                            parameters_type: {
                                let mut t = Vec::new();
                                for p in parameters_type {
                                    self.check_type(p)?;
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
                    self.check_type(element_type)?;
                    if let Some(len_expr) = len_expr {
                        self.visit_expr(Rc::clone(&len_expr))?;
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
                            return Err(Diagnostic {
                                span: r#type.borrow().span,
                                kind: DiagnosticKind::Error,
                                message: format!(
                                    "'restrict' requires an array or a pointer point to an object type"
                                ),
                                notes: vec![],
                            });
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
                } => self.visit_expr(Rc::clone(expr))?,
                _ => {}
            }
        }

        Ok(())
    }

    pub fn visit_declaration(
        &mut self,
        node: Rc<RefCell<Declaration<'a>>>,
    ) -> Result<(), Diagnostic<'a>> {
        self.contexts
            .push(Context::Decl(node.borrow().kind.clone()));

        {
            let mut node = node.borrow_mut();
            self.check_type(&mut node.r#type)?;

            match &node.kind {
                DeclarationKind::Parameter => {
                    node.r#type = as_parameter_type(Rc::clone(&node.r#type));
                }
                _ => {}
            }
        }

        let node = node.borrow();

        if node.storage_classes.len() > 1 {
            return Err(Diagnostic {
                span: node.span,
                kind: DiagnosticKind::Error,
                message: format!("at most one storage class specifier is allowed"),
                notes: vec![],
            });
        }

        if node.r#type.borrow().has_alignas() {
            for storage_class in &node.storage_classes {
                match &storage_class.kind {
                    StorageClassKind::Register => {
                        return Err(Diagnostic {
                            span: storage_class.span,
                            kind: DiagnosticKind::Error,
                            message: format!(
                                "alignas cannot be applied to an object with 'register' storage class specifier"
                            ),
                            notes: vec![],
                        });
                    }
                    StorageClassKind::Typedef => {
                        return Err(Diagnostic {
                            span: storage_class.span,
                            kind: DiagnosticKind::Error,
                            message: format!("alignas cannot be applied to a type"),
                            notes: vec![],
                        });
                    }
                    _ => {}
                }
            }
        }

        //node.name.len()==0时, 并没有声明任何变量, 但仍要进行类型检查

        for child in &node.children {
            self.visit_declaration(Rc::clone(child))?;
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

                self.enter_scope();
                self.func_symtabs.push(Rc::clone(&self.cur_symtab));
                self.funcs.push(Rc::clone(&node.r#type));

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
                    match &node.r#type.borrow().kind {
                        TypeKind::Function {
                            return_type,
                            parameters_type,
                            ..
                        } => {
                            if !return_type.borrow().is_void()
                                && !return_type.borrow().is_complete()
                            {
                                return Err(Diagnostic {
                                    span: return_type.borrow().span,
                                    kind: DiagnosticKind::Error,
                                    message: format!("return type must be complete"),
                                    notes: vec![],
                                });
                            }
                            for parameter_type in parameters_type {
                                if !parameter_type.borrow().is_void()//这种情况检查参数声明时已经处理过了
                                    && !parameter_type.borrow().is_complete()
                                {
                                    return Err(Diagnostic {
                                        span: parameter_type.borrow().span,
                                        kind: DiagnosticKind::Error,
                                        message: format!("parameter type must be complete"),
                                        notes: vec![],
                                    });
                                }
                            }
                        }
                        _ => {}
                    }
                    self.visit_stmt(Rc::clone(body))?;
                }

                self.funcs.pop();
                self.func_symtabs.pop();
                self.leave_scope();
            }
            DeclarationKind::Parameter => {
                if node.r#type.borrow().has_alignas() {
                    return Err(Diagnostic {
                        span: node.span,
                        kind: DiagnosticKind::Error,
                        message: format!("alignas cannot be applied to a parameter"),
                        notes: vec![],
                    });
                }
                for storage_class in &node.storage_classes {
                    if storage_class.kind != StorageClassKind::Register {
                        return Err(Diagnostic {
                            span: storage_class.span,
                            kind: DiagnosticKind::Error,
                            message: format!(
                                "only 'register' storage class specifier is allowed in parameter"
                            ),
                            notes: vec![],
                        });
                    }
                }
                if node.name.len() > 0 && node.r#type.borrow().is_void() {
                    return Err(Diagnostic {
                        span: node.span,
                        kind: DiagnosticKind::Error,
                        message: format!("parameter cannot have a void type"),
                        notes: vec![],
                    });
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
                    //在处理完成员声明前保持不完整(members为None说明不完整)
                    let member_symtab = Rc::new(RefCell::new(SymbolTable::new()));
                    self.member_symtabs.push(Rc::clone(&member_symtab));
                    for decl in members_decl {
                        self.visit_declaration(Rc::clone(&decl))?;
                    }
                    self.member_symtabs.pop();

                    match &mut node.r#type.borrow_mut().kind {
                        //这里members一定为None, 否则在之前加入符号表时就会报错
                        TypeKind::Record { members, .. } => {
                            *members = Some(
                                member_symtab
                                    .borrow()
                                    .namespaces
                                    .get(&Namespace::Member)
                                    .unwrap_or(&IndexMap::new())
                                    .clone(),
                            );
                        }
                        _ => {}
                    }
                }
            }
            DeclarationKind::Member { bit_field } => {
                if let Some(_) = bit_field
                    && node.r#type.borrow().has_alignas()
                {
                    return Err(Diagnostic {
                        span: node.span,
                        kind: DiagnosticKind::Error,
                        message: format!("alignas cannot be applied to a member with bit-field"),
                        notes: vec![],
                    });
                }

                if node.storage_classes.len() > 0 {
                    return Err(Diagnostic {
                        span: node.storage_classes[0].span,
                        kind: DiagnosticKind::Error,
                        message: format!("member should not hava any storage class specifier"),
                        notes: vec![],
                    });
                }

                if !node.r#type.borrow().is_complete() {
                    return Err(Diagnostic {
                        span: node.span,
                        kind: DiagnosticKind::Error,
                        message: format!("'{}' is not complete", node.r#type.borrow().to_string()),
                        notes: vec![],
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
                                Variant::Int(value) => Some(value.clone()),
                                _ => {
                                    return Err(Diagnostic {
                                        span: t.borrow().span,
                                        kind: DiagnosticKind::Error,
                                        message: format!("bit-field must has an integer type"),
                                        notes: vec![],
                                    });
                                }
                            },
                            None => None,
                        },
                    },
                    r#type: Rc::clone(&node.r#type),
                    attributes: node.attributes.clone(),
                }));

                self.member_symtabs
                    .last()
                    .unwrap()
                    .borrow_mut()
                    .add(Namespace::Member, symbol)?;
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
                        enum_consts,
                        ..
                    } = &mut node.r#type.borrow_mut().kind
                    else {
                        unreachable!()
                    };

                    if let None = enum_consts {
                        *enum_consts = Some(IndexMap::new());
                    }
                    let enum_consts = enum_consts.as_ref().unwrap();

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
