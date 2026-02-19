use super::TypeChecker;
use crate::{
    ast::decl::{Declaration, DeclarationKind, StorageClassKind},
    ctype::{RecordKind, Type, TypeKind, as_parameter_type},
    diagnostic::warning,
    preprocessor::expressions::has_c_attribute,
    symtab::{Namespace, Symbol, SymbolKind, SymbolTable},
    typechecker::Context,
    variant::Variant,
};
use codespan_reporting::diagnostic::{Diagnostic, Label};
use indexmap::IndexMap;
use num::{BigInt, ToPrimitive};
use std::{cell::RefCell, ptr::addr_of, rc::Rc};

impl TypeChecker {
    pub fn visit_declaration(
        &mut self,
        node: Rc<RefCell<Declaration>>,
    ) -> Result<(), Diagnostic<usize>> {
        self.contexts
            .push(Context::Decl(node.borrow().kind.clone()));

        {
            let mut node = node.borrow_mut();
            match &node.kind {
                DeclarationKind::Record { .. } | DeclarationKind::Enum { .. } => {}
                _ => self.check_type(&mut node.r#type)?,
            }

            match &node.kind {
                DeclarationKind::Parameter => {
                    node.r#type = as_parameter_type(Rc::clone(&node.r#type));
                }
                _ => {}
            }
        }

        let mut node = node.borrow_mut();

        //统一检查storage class

        if node
            .storage_classes
            .iter()
            .any(|x| x.kind == StorageClassKind::ThreadLocal)
        {
            if node.storage_classes.len() > 2 {
                return Err(Diagnostic::error()
                    .with_message(format!(
                        "at most two storage classes specifier with 'thread_local' is allowed"
                    ))
                    .with_label(Label::primary(
                        node.storage_classes[2].file_id,
                        node.storage_classes[2].span,
                    )));
            } else if node.storage_classes.len() == 2
                && let Some(t) = node.storage_classes.iter().find(|x| {
                    x.kind != StorageClassKind::Static
                        && x.kind != StorageClassKind::Extern
                        && x.kind != StorageClassKind::ThreadLocal
                })
            {
                return Err(Diagnostic::error()
                    .with_message(format!("'thread_local' storage class specifier can only combine with 'static' or 'extern'"))
                    .with_label(Label::primary(t.file_id, t.span)));
            }
        } else if node.storage_classes.len() > 1 {
            return Err(Diagnostic::error()
                .with_message(format!("at most one storage class specifier is allowed"))
                .with_label(Label::primary(
                    node.storage_classes[1].file_id,
                    node.storage_classes[1].span,
                )));
        }

        if let Some(t) = node
            .storage_classes
            .iter()
            .find(|x| x.kind == StorageClassKind::Auto)
        {
            if !matches!(node.kind, DeclarationKind::Var { .. }) {
                return Err(Diagnostic::error()
                    .with_message("'auto' can only apply to an object (except a parameter)")
                    .with_label(Label::primary(t.file_id, t.span)));
            } else if let None = self.cur_symtab.borrow().parent {
                return Err(Diagnostic::error()
                    .with_message("'auto' can only be used at the file scope")
                    .with_label(Label::primary(t.file_id, t.span)));
            }
        }

        if let Some(t) = node
            .storage_classes
            .iter()
            .find(|x| x.kind == StorageClassKind::Register)
        {
            if !matches!(node.kind, DeclarationKind::Var { .. })
                && !matches!(node.kind, DeclarationKind::Parameter)
            {
                return Err(Diagnostic::error()
                    .with_message("'register' can only apply to an object or a parameter")
                    .with_label(Label::primary(t.file_id, t.span)));
            } else if let None = self.cur_symtab.borrow().parent {
                return Err(Diagnostic::error()
                    .with_message("'register' can only be used at the file scope")
                    .with_label(Label::primary(t.file_id, t.span)));
            }
        }

        if let Some(t) = node
            .storage_classes
            .iter()
            .find(|x| x.kind == StorageClassKind::Static)
        {
            if !matches!(node.kind, DeclarationKind::Var { .. })
                && !matches!(node.kind, DeclarationKind::Function { .. })
            {
                return Err(Diagnostic::error()
                    .with_message(
                        "'static' can only apply to an object (except a parameter) or a function",
                    )
                    .with_label(Label::primary(t.file_id, t.span)));
            } else if node.r#type.borrow().is_vla() {
                return Err(Diagnostic::error()
                    .with_message("vla cannot have static duration")
                    .with_label(Label::primary(t.file_id, t.span)));
            }
        }

        if let Some(t) = node
            .storage_classes
            .iter()
            .find(|x| x.kind == StorageClassKind::Extern)
        {
            if !matches!(node.kind, DeclarationKind::Var { .. })
                && !matches!(node.kind, DeclarationKind::Function { .. })
            {
                return Err(Diagnostic::error()
                    .with_message(
                        "'extern' can only apply to an object (except a parameter) or a function",
                    )
                    .with_label(Label::primary(t.file_id, t.span)));
            } else if node.r#type.borrow().is_vla() {
                return Err(Diagnostic::error()
                    .with_message("vla cannot have linkage")
                    .with_label(Label::primary(t.file_id, t.span)));
            }
        }

        if let Some(t) = node
            .storage_classes
            .iter()
            .find(|x| x.kind == StorageClassKind::ThreadLocal)
        {
            if matches!(node.kind, DeclarationKind::Function { .. }) {
                return Err(Diagnostic::error()
                    .with_message("'thread_local' cannot apply to function")
                    .with_label(Label::primary(t.file_id, t.span)));
            } else if !node
                .storage_classes
                .iter()
                .any(|x| x.kind == StorageClassKind::Static || x.kind == StorageClassKind::Extern)
            {
                return Err(Diagnostic::error()
                    .with_message("'thread_local' must be with 'static' or 'extern'")
                    .with_label(Label::primary(t.file_id, t.span)));
            }
        }

        //检查storage class完成

        if node.r#type.borrow().has_alignas() {
            for storage_class in &node.storage_classes {
                match &storage_class.kind {
                    StorageClassKind::Register => {
                        return Err(Diagnostic::error().with_message(format!(
                                "alignas cannot be applied to an object with 'register' storage class specifier"
                            )).with_label(Label::primary(storage_class.file_id, storage_class.span)));
                    }
                    StorageClassKind::Typedef => {
                        return Err(Diagnostic::error()
                            .with_message(format!("alignas cannot be applied to a type"))
                            .with_label(Label::primary(
                                storage_class.file_id,
                                storage_class.span,
                            )));
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
                let symbol = Rc::new(RefCell::new(Symbol {
                    define_loc: match initializer {
                        Some(_) => Some((node.file_id, node.span)),
                        None => None,
                    },
                    declare_locs: vec![(node.file_id, node.span)],
                    name: node.name.clone(),
                    kind: SymbolKind::Object {
                        storage_classes: node.storage_classes.clone(),
                        init_value: Variant::Unknown,
                    },
                    r#type: Rc::clone(&node.r#type),
                    attributes: node.attributes.clone(),
                }));

                self.cur_symtab
                    .borrow_mut()
                    .add(Namespace::Ordinary, Rc::clone(&symbol))?;

                if let Some(initializer) = initializer {
                    self.visit_initializer(Rc::clone(initializer), Rc::clone(&node.r#type))?;

                    if node.r#type.borrow().is_const() {
                        let SymbolKind::Object { init_value, .. } = &mut symbol.borrow_mut().kind
                        else {
                            unreachable!();
                        };
                        *init_value = initializer.borrow().value.clone();
                    }
                }
            }
            DeclarationKind::Function {
                parameter_decls,
                function_specs,
                body,
                ..
            } => {
                let parent_symtab = Rc::clone(&self.cur_symtab);

                self.enter_scope();
                self.func_symtabs.push(Rc::clone(&self.cur_symtab));
                self.func_types.push(Rc::clone(&node.r#type));

                for decl in parameter_decls {
                    self.visit_declaration(Rc::clone(decl))?;
                }

                parent_symtab.borrow_mut().add(
                    Namespace::Ordinary,
                    Rc::new(RefCell::new(Symbol {
                        define_loc: match body {
                            Some(_) => Some((node.file_id, node.span)),
                            None => None,
                        },
                        declare_locs: vec![(node.file_id, node.span)],
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
                                return Err(Diagnostic::error()
                                    .with_message(format!("return type must be complete"))
                                    .with_label(Label::primary(
                                        return_type.borrow().file_id,
                                        return_type.borrow().span,
                                    )));
                            }
                            for parameter_type in parameters_type {
                                if !parameter_type.borrow().is_void()//这种情况检查参数声明时已经处理过了
                                    && !parameter_type.borrow().is_complete()
                                {
                                    return Err(Diagnostic::error()
                                        .with_message(format!("parameter type must be complete"))
                                        .with_label(Label::primary(
                                            parameter_type.borrow().file_id,
                                            parameter_type.borrow().span,
                                        )));
                                }
                            }
                        }
                        _ => {}
                    }
                    self.visit_stmt(Rc::clone(body))?;
                }

                self.func_types.pop();
                self.func_symtabs.pop();
                match &mut node.kind {
                    DeclarationKind::Function { symtab, .. } => {
                        *symtab = Some(self.leave_scope());
                    }
                    _ => {
                        self.leave_scope();
                    }
                }
            }
            DeclarationKind::Parameter => {
                if node.r#type.borrow().has_alignas() {
                    return Err(Diagnostic::error()
                        .with_message(format!("alignas cannot be applied to a parameter"))
                        .with_label(Label::primary(node.file_id, node.span)));
                }
                if node.name.len() > 0 && node.r#type.borrow().is_void() {
                    return Err(Diagnostic::error()
                        .with_message(format!("parameter cannot have a void type"))
                        .with_label(Label::primary(node.file_id, node.span)));
                }
                let index = match self
                    .cur_symtab
                    .borrow()
                    .namespaces
                    .get(&Namespace::Ordinary)
                {
                    Some(t) => t
                        .iter()
                        .map(|(_, symbol)| {
                            if let SymbolKind::Parameter { .. } = symbol.borrow().kind {
                                1
                            } else {
                                0
                            }
                        })
                        .sum(),
                    None => 0,
                };
                self.cur_symtab.borrow_mut().add(
                    Namespace::Ordinary,
                    Rc::new(RefCell::new(Symbol {
                        define_loc: Some((node.file_id, node.span)),
                        declare_locs: vec![(node.file_id, node.span)],
                        name: node.name.clone(),
                        kind: SymbolKind::Parameter {
                            storage_classes: node.storage_classes.clone(),
                            index,
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
                        define_loc: Some((node.file_id, node.span)),
                        declare_locs: vec![(node.file_id, node.span)],
                        name: node.name.clone(),
                        kind: SymbolKind::Type,
                        r#type: Rc::new(RefCell::new(Type {
                            kind: TypeKind::Typedef {
                                name: node.name.clone(),
                                r#type: Some(Rc::clone(&node.r#type)),
                            },
                            ..Type::new(node.file_id, node.span)
                        })),
                        attributes: node.attributes.clone(),
                    })),
                )?;
            }
            DeclarationKind::Record { members_decl } => {
                self.cur_symtab.borrow_mut().add(
                    Namespace::Tag,
                    Rc::new(RefCell::new(Symbol {
                        define_loc: match members_decl {
                            Some(_) => Some((node.file_id, node.span)),
                            None => None,
                        },
                        declare_locs: vec![(node.file_id, node.span)],
                        name: node.name.clone(),
                        r#type: Rc::clone(&node.r#type),
                        kind: SymbolKind::Record {
                            kind: if node.r#type.borrow().is_union() {
                                RecordKind::Union
                            } else {
                                RecordKind::Struct
                            },
                        },
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
                let bit_field = bit_field.clone();
                if let Some(_) = bit_field
                    && node.r#type.borrow().has_alignas()
                {
                    return Err(Diagnostic::error()
                        .with_message(format!(
                            "alignas cannot be applied to a member with bit-field"
                        ))
                        .with_label(Label::primary(node.file_id, node.span)));
                }

                if !node.r#type.borrow().is_complete() {
                    return Err(Diagnostic::error()
                        .with_message(format!(
                            "'{}' is not complete",
                            node.r#type.borrow().to_string()
                        ))
                        .with_label(Label::primary(node.file_id, node.span)));
                }

                if let Some(_) = bit_field
                    && !node.r#type.borrow().is_integer()
                {
                    return Err(Diagnostic::error()
                        .with_message("bit-field must be an integer")
                        .with_label(Label::primary(node.file_id, node.span)));
                }

                if let Some(t) = &bit_field {
                    self.visit_expr(Rc::clone(t))?;
                    if node.name.len() == 0 {
                        //无名位域会影响内存结构, 但无名非位域不会
                        node.name = format!("{}", addr_of!(*node) as usize);
                    }
                }

                let symbol = Rc::new(RefCell::new(Symbol {
                    define_loc: Some((node.file_id, node.span)),
                    declare_locs: vec![(node.file_id, node.span)],
                    name: node.name.clone(),
                    kind: SymbolKind::Member {
                        bit_field: match bit_field {
                            Some(t) => match &t.borrow().value {
                                Variant::Int(value) => match value.to_usize() {
                                    Some(value) => Some(value),
                                    None => {
                                        return Err(Diagnostic::error()
                                            .with_message(format!(
                                                "the width of bit-field is too large "
                                            ))
                                            .with_label(Label::primary(
                                                t.borrow().file_id,
                                                t.borrow().span,
                                            )));
                                    }
                                },
                                _ => {
                                    return Err(Diagnostic::error()
                                        .with_message(format!(
                                            "the width of bit-field must has an integer type"
                                        ))
                                        .with_label(Label::primary(
                                            t.borrow().file_id,
                                            t.borrow().span,
                                        )));
                                }
                            },
                            None => None,
                        },
                        index: 0,
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
                        define_loc: match enumerators {
                            Some(_) => Some((node.file_id, node.span)),
                            None => None,
                        },
                        declare_locs: vec![(node.file_id, node.span)],
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
                            attributes: underlying.borrow().attributes.clone(),
                            kind,
                            ..Type::new(underlying.borrow().file_id, underlying.borrow().span)
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
                    define_loc: Some((node.file_id, node.span)),
                    declare_locs: vec![(node.file_id, node.span)],
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
            DeclarationKind::StaticAssert { expr } => {
                self.visit_expr(Rc::clone(expr))?;
                match &expr.borrow().value {
                    Variant::Int(value) => {
                        if *value == BigInt::ZERO {
                            return Err(Diagnostic::error()
                                .with_message(format!("static assertion failed: {}", node.name))
                                .with_label(Label::primary(node.file_id, node.span)));
                        }
                    }
                    _ => {
                        return Err(Diagnostic::error()
                            .with_message("static assertion expresion must be an integer constant")
                            .with_label(Label::primary(
                                expr.borrow().file_id,
                                expr.borrow().span,
                            )));
                    }
                }
            }
            DeclarationKind::Attribute => {
                for attribute in &node.attributes {
                    let prefix_name = attribute.borrow().prefix_name.clone();
                    let name = attribute.borrow().name.clone();
                    if has_c_attribute(prefix_name, name.clone()) == 0 {
                        warning(
                            format!("unknown attribute '{name}'"),
                            attribute.borrow().file_id,
                            attribute.borrow().span,
                            vec![],
                        );
                    }
                }
            }
        }

        self.contexts.pop();
        Ok(())
    }
}
