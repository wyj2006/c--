use super::{CParser, Rule};
use crate::ast::decl::{
    Declaration, DeclarationKind, FunctionSpec, FunctionSpecKind, StorageClass, StorageClassKind,
};
use crate::ast::expr::ExprKind;
use crate::ast::{Attribute, AttributeKind};
use crate::ctype::TypeQual;
use crate::ctype::{RecordKind, Type, TypeKind};
use crate::diagnostic::from_pest_span;
use crate::parser::parse_identifier;
use codespan_reporting::diagnostic::{Diagnostic, Label};
use pest::iterators::Pair;
use std::cell::RefCell;
use std::rc::Rc;

impl CParser {
    //删除decls中record/enum的非定义和非前向声明
    pub fn filter_decls(
        &self,
        mut decls: Vec<Rc<RefCell<Declaration>>>,
    ) -> Vec<Rc<RefCell<Declaration>>> {
        let mut may_forward_decl = true;
        for decl in &decls {
            match decl.borrow().kind {
                DeclarationKind::Enum { .. } | DeclarationKind::Record { .. } => {}
                _ => {
                    may_forward_decl = false;
                    break;
                }
            }
        }

        let mut i = 0;
        while i < decls.len() {
            let decl = Rc::clone(&decls[i]);
            match &decl.borrow().kind {
                DeclarationKind::Record { members_decl: None }
                | DeclarationKind::Enum { enumerators: None }
                    if !may_forward_decl =>
                {
                    decls.remove(i);
                }
                _ => i += 1,
            }
        }

        decls
    }

    pub fn parse_function_definition(
        &self,
        rule: Pair<Rule>,
    ) -> Result<Vec<Rc<RefCell<Declaration>>>, Diagnostic<usize>> {
        let mut decls = Vec::new();
        let mut attributes = Vec::new();
        let mut type_attrs = Vec::new();
        let mut spec_type = None;
        let mut storage_classes = Vec::new();
        let mut function_specs = Vec::new();
        let mut function_decl = None;
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::attribute_specifier_sequence => {
                    attributes.extend(self.parse_attribute_specifier_sequence(rule)?);
                }
                Rule::declaration_specifiers_without_typedef
                | Rule::declaration_specifiers_with_typedef => {
                    match self.parse_declaration_specifiers(rule)? {
                        (
                            r#type,
                            extern_storage_classes,
                            extern_function_specs,
                            spec_decls,
                            extern_type_attrs,
                            extern_var_attrs,
                        ) => {
                            spec_type = Some(r#type);
                            storage_classes = extern_storage_classes;
                            function_specs = extern_function_specs;
                            decls.extend(spec_decls);
                            type_attrs.extend(extern_type_attrs);
                            attributes.extend(extern_var_attrs);
                        }
                    }
                }
                Rule::declarator => {
                    let span = rule.as_span();
                    let decl = self.parse_declarator(spec_type.clone().unwrap(), rule)?;
                    decl.borrow_mut()
                        .r#type
                        .borrow_mut()
                        .attributes
                        .extend(type_attrs.clone());
                    decl.borrow_mut().attributes.extend(attributes.clone());
                    decl.borrow_mut()
                        .storage_classes
                        .extend(storage_classes.clone());
                    if let DeclarationKind::Function {
                        function_specs: t, ..
                    } = &mut decl.borrow_mut().kind
                    {
                        t.extend(function_specs.clone());
                    } else {
                        return Err(Diagnostic::error()
                            .with_message(format!("not a function"))
                            .with_label(Label::primary(self.file_id, from_pest_span(span))));
                    }
                    function_decl = Some(decl);
                }
                Rule::compound_statement => {
                    if let Some(function_decl) = &function_decl {
                        if let DeclarationKind::Function { body, .. } =
                            &mut function_decl.borrow_mut().kind
                        {
                            *body = Some(self.parse_compound_statement(rule)?);
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
        decls.push(function_decl.unwrap());
        Ok(self.filter_decls(decls))
    }

    pub fn parse_declaration(
        &self,
        rule: Pair<Rule>,
    ) -> Result<Vec<Rc<RefCell<Declaration>>>, Diagnostic<usize>> {
        let span = rule.as_span();
        let mut decls = Vec::new();
        //对象的属性
        let mut attributes = Vec::new();
        //对象的类型的属性
        let mut type_attrs = Vec::new();
        let mut spec_type = None;
        let mut storage_classes = Vec::new();
        let mut function_specs = Vec::new();
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::attribute_specifier_sequence => {
                    attributes.extend(self.parse_attribute_specifier_sequence(rule)?);
                }
                Rule::declaration_specifiers_without_typedef
                | Rule::declaration_specifiers_with_typedef => {
                    match self.parse_declaration_specifiers(rule)? {
                        (
                            r#type,
                            extern_storage_classes,
                            extern_function_specs,
                            spec_decls,
                            extern_type_attrs,
                            extern_var_attrs,
                        ) => {
                            spec_type = Some(r#type);
                            storage_classes = extern_storage_classes;
                            function_specs = extern_function_specs;
                            decls.extend(spec_decls);
                            type_attrs.extend(extern_type_attrs);
                            attributes.extend(extern_var_attrs);
                        }
                    }
                }
                Rule::initializer => {
                    let initializer = self.parse_initializer(rule)?;
                    let decl = decls.last_mut().unwrap();
                    let mut decl_borrow = decl.borrow_mut();
                    if let DeclarationKind::Var { initializer: old } = &mut decl_borrow.kind {
                        *old = Some(initializer);
                    } else {
                        return Err(Diagnostic::error()
                            .with_message(format!("only variables can be initialized"))
                            .with_label(Label::primary(
                                initializer.borrow().file_id,
                                initializer.borrow().span,
                            )));
                    }
                    decl_borrow.attributes.extend(attributes.clone());
                }
                Rule::declarator | Rule::abstract_declarator => {
                    let decl = self.parse_declarator(spec_type.clone().unwrap(), rule)?;
                    decl.borrow_mut()
                        .r#type
                        .borrow_mut()
                        .attributes
                        .extend(type_attrs.clone());
                    decl.borrow_mut().attributes.extend(attributes.clone());
                    decl.borrow_mut()
                        .storage_classes
                        .extend(storage_classes.clone());
                    if storage_classes
                        .iter()
                        .any(|x| x.kind == StorageClassKind::Typedef)
                    {
                        decl.borrow_mut().kind = DeclarationKind::Type;
                    }
                    if let DeclarationKind::Function {
                        function_specs: t, ..
                    } = &mut decl.borrow_mut().kind
                    {
                        t.extend(function_specs.clone());
                    } else if function_specs.len() > 0 {
                        return Err(Diagnostic::error()
                            .with_message(format!(
                                "function specifier can only appear on functions"
                            ))
                            .with_label(Label::primary(
                                function_specs[0].file_id,
                                function_specs[0].span,
                            )));
                    }
                    decls.push(decl);
                }
                Rule::static_assert_declaration => {
                    decls.push(self.parse_static_assert_declaration(rule)?)
                }
                Rule::attribute_declaration => {
                    let span = rule.as_span();
                    for rule in rule.into_inner() {
                        match rule.as_rule() {
                            Rule::attribute_specifier_sequence => {
                                attributes.extend(self.parse_attribute_specifier_sequence(rule)?)
                            }
                            _ => unreachable!(),
                        }
                    }
                    decls.push(Rc::new(RefCell::new(Declaration {
                        attributes: attributes.clone(),
                        ..Declaration::new(
                            self.file_id,
                            from_pest_span(span),
                            DeclarationKind::Attribute,
                        )
                    })))
                }
                _ => unreachable!(),
            }
        }
        if let Some(mut spec_type) = spec_type
            && decls.len() == 0
        {
            //没有declarator时的情况
            decls.push(Rc::new(RefCell::new(Declaration {
                attributes,
                r#type: Rc::new(RefCell::new({
                    spec_type.attributes.extend(type_attrs);
                    spec_type
                })),
                storage_classes,
                ..Declaration::new(
                    self.file_id,
                    from_pest_span(span),
                    DeclarationKind::Var { initializer: None },
                )
            })));
        }
        Ok(self.filter_decls(decls))
    }

    pub fn parse_declaration_specifiers(
        &self,
        rule: Pair<Rule>,
    ) -> Result<
        (
            //因为解析出来的不是最终的类型, 所以不需要引用计数
            Type,
            Vec<StorageClass>,
            Vec<FunctionSpec>,
            //可能有record或enum的定义
            Vec<Rc<RefCell<Declaration>>>,
            //应用于类型的属性, 就可能有AlignAs一种属性
            Vec<Rc<RefCell<Attribute>>>,
            //应用于声明对象的属性, 可能是Noreturn
            Vec<Rc<RefCell<Attribute>>>,
        ),
        Diagnostic<usize>,
    > {
        let span = rule.as_span();
        let mut attributes = Vec::new();
        let mut extern_type_attrs = Vec::new();
        let mut extern_var_attrs = Vec::new();
        let mut types = Vec::new();
        let mut qualifiers = Vec::new();
        let mut storage_classes = Vec::new();
        let mut function_specs = Vec::new();
        let mut decls = Vec::new();
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::type_qualifier => qualifiers.push(self.parse_type_qualifier(rule)?),
                Rule::storage_class_specifier => match self.parse_storage_class_specifier(rule)? {
                    t @ StorageClass {
                        file_id,
                        kind,
                        span,
                    } if matches!(kind, StorageClassKind::Auto) => {
                        types.push(Type {
                            kind: TypeKind::Auto(None),
                            ..Type::new(file_id, span)
                        });
                        storage_classes.push(t);
                    }
                    t => storage_classes.push(t),
                },
                Rule::function_specifier => match rule.as_str() {
                    "inline" => function_specs.push(FunctionSpec::new(
                        self.file_id,
                        from_pest_span(span),
                        FunctionSpecKind::Inline,
                    )),
                    "_Noreturn" => {
                        extern_var_attrs.push(Rc::new(RefCell::new(Attribute {
                            name: "noreturn".to_string(),
                            kind: AttributeKind::Noreturn,
                            ..Attribute::new(self.file_id, from_pest_span(span))
                        })));
                    }
                    _ => unreachable!(),
                },
                Rule::type_specifier => match self.parse_type_specifier(rule)? {
                    (r#type, spec_decls) => {
                        types.push(r#type);
                        decls.extend(spec_decls);
                    }
                },
                Rule::attribute_specifier_sequence => {
                    attributes.extend(self.parse_attribute_specifier_sequence(rule)?);
                }
                Rule::alignment_specifier => {
                    let span = rule.as_span();
                    let mut expr = None;
                    let mut r#type = None;
                    for rule in rule.into_inner() {
                        match rule.as_rule() {
                            Rule::constant_expression => {
                                expr = Some(self.parse_constant_expression(rule)?)
                            }
                            Rule::type_name => match self.parse_type_name(rule)? {
                                (type_name_type, type_name_decls) => {
                                    decls.extend(type_name_decls);
                                    r#type = Some(type_name_type);
                                }
                            },
                            _ => unreachable!(),
                        }
                    }
                    extern_type_attrs.push(Rc::new(RefCell::new(Attribute {
                        name: "alignas".to_string(),
                        kind: AttributeKind::AlignAs { r#type, expr },
                        ..Attribute::new(self.file_id, from_pest_span(span))
                    })));
                }
                Rule::typedef_name => {
                    let span = rule.as_span();
                    for rule in rule.into_inner() {
                        if let Rule::identifier = rule.as_rule() {
                            types.push(Type {
                                kind: TypeKind::Typedef {
                                    name: parse_identifier(rule.as_str())?,
                                    r#type: None,
                                },
                                ..Type::new(self.file_id, from_pest_span(span))
                            });
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
        //排序, 方便后面组合
        types.sort_by_key(|x| match x.kind {
            TypeKind::Char
            | TypeKind::Int
            | TypeKind::Complex(..)
            | TypeKind::BitInt {
                unsigned: _,
                width_expr: _,
            } => 0,
            TypeKind::Double | TypeKind::Float | TypeKind::Short => 1,
            TypeKind::Long => 2,
            //对应 signed unsigned
            TypeKind::Signed | TypeKind::Unsigned => 3,
            _ => 4,
        });
        //开始组合类型
        if types.len() == 0 {
            return Err(Diagnostic::error()
                .with_message(format!("type specifiers missing"))
                .with_label(Label::primary(self.file_id, from_pest_span(span))));
        }
        let init_type = Ok(types.remove(0));
        let mut final_type = types.iter().fold(init_type, |acc, x| {
            if let Err(_) = acc {
                return acc;
            }
            let mut acc = acc.unwrap();
            match x.kind {
                TypeKind::Float => match acc.kind {
                    TypeKind::Complex(None) => {
                        acc.kind = TypeKind::Complex(Some(Rc::new(RefCell::new(x.clone()))));
                        return Ok(acc);
                    }
                    _ => {}
                },
                TypeKind::Double => match acc.kind {
                    TypeKind::Complex(None) => {
                        acc.kind = TypeKind::Complex(Some(Rc::new(RefCell::new(x.clone()))));
                        return Ok(acc);
                    }
                    _ => {}
                },
                TypeKind::Short => match acc.kind {
                    TypeKind::Int => {
                        acc.kind = TypeKind::Short;
                        return Ok(acc);
                    }
                    _ => {}
                },
                TypeKind::Long => match acc.kind.clone() {
                    TypeKind::Int => {
                        acc.kind = TypeKind::Long;
                        return Ok(acc);
                    }
                    TypeKind::Long => {
                        acc.kind = TypeKind::LongLong;
                        return Ok(acc);
                    }
                    TypeKind::Double => {
                        acc.kind = TypeKind::LongDouble;
                        return Ok(acc);
                    }
                    TypeKind::Complex(Some(t)) => {
                        if let TypeKind::Double = t.borrow().kind {
                            acc.kind = TypeKind::Complex(Some(Rc::new(RefCell::new(Type {
                                attributes: x.attributes.clone(),
                                kind: TypeKind::LongDouble,
                                ..Type::new(self.file_id, x.span)
                            }))));
                            return Ok(acc);
                        }
                    }
                    _ => {}
                },
                TypeKind::Signed => match acc.kind {
                    TypeKind::Char => {
                        acc.kind = TypeKind::SignedChar;
                        return Ok(acc);
                    }
                    TypeKind::Int => {
                        acc.kind = TypeKind::Int;
                        return Ok(acc);
                    }
                    TypeKind::Short => {
                        acc.kind = TypeKind::Short;
                        return Ok(acc);
                    }
                    TypeKind::Long => {
                        acc.kind = TypeKind::Long;
                        return Ok(acc);
                    }
                    TypeKind::LongLong => {
                        acc.kind = TypeKind::LongLong;
                        return Ok(acc);
                    }
                    TypeKind::BitInt {
                        unsigned: _,
                        width_expr,
                    } => {
                        acc.kind = TypeKind::BitInt {
                            unsigned: false,
                            width_expr: width_expr,
                        };
                        return Ok(acc);
                    }
                    _ => {
                        acc.kind = TypeKind::Int;
                        return Ok(acc);
                    }
                },
                TypeKind::Unsigned => match acc.kind {
                    TypeKind::Char => {
                        acc.kind = TypeKind::UnsignedChar;
                        return Ok(acc);
                    }
                    TypeKind::Int => {
                        acc.kind = TypeKind::UInt;
                        return Ok(acc);
                    }
                    TypeKind::Short => {
                        acc.kind = TypeKind::UShort;
                        return Ok(acc);
                    }
                    TypeKind::Long => {
                        acc.kind = TypeKind::ULong;
                        return Ok(acc);
                    }
                    TypeKind::LongLong => {
                        acc.kind = TypeKind::ULongLong;
                        return Ok(acc);
                    }
                    TypeKind::BitInt {
                        unsigned: _,
                        width_expr,
                    } => {
                        acc.kind = TypeKind::BitInt {
                            unsigned: true,
                            width_expr: width_expr,
                        };
                        return Ok(acc);
                    }
                    _ => {
                        acc.kind = TypeKind::UInt;
                        return Ok(acc);
                    }
                },
                _ => {}
            }
            Err(Diagnostic::error()
                .with_message(format!("cannot combine {} and {}", acc, x))
                .with_label(Label::primary(x.file_id, x.span)))
        })?;

        let atomic_index = qualifiers.iter().position(|x| {
            if let TypeQual::Atomic = x {
                true
            } else {
                false
            }
        });
        if let Some(atomic_index) = atomic_index {
            final_type = Type {
                attributes: attributes.clone(),
                kind: TypeKind::Atomic(Rc::new(RefCell::new(final_type))),
                ..Type::new(self.file_id, from_pest_span(span))
            };
            qualifiers.remove(atomic_index);
        }
        if qualifiers.len() > 0 {
            final_type = Type {
                attributes,
                kind: TypeKind::Qualified {
                    qualifiers,
                    r#type: Rc::new(RefCell::new(final_type)),
                },
                ..Type::new(self.file_id, from_pest_span(span))
            };
        } else {
            final_type.span = from_pest_span(span);
            final_type.attributes.extend(attributes);
        }
        Ok((
            final_type,
            storage_classes,
            function_specs,
            decls,
            extern_type_attrs,
            extern_var_attrs,
        ))
    }

    pub fn parse_type_specifier(
        &self,
        rule: Pair<Rule>,
    ) -> Result<(Type, Vec<Rc<RefCell<Declaration>>>), Diagnostic<usize>> {
        let span = rule.as_span();
        let mut decls = Vec::new();
        let kind = match rule.as_str() {
            "void" => TypeKind::Void,
            "char" => TypeKind::Char,
            "short" => TypeKind::Short,
            "int" => TypeKind::Int,
            "signed" => TypeKind::Signed,
            "long" => TypeKind::Long,
            "unsigned" => TypeKind::Unsigned,
            "float" => TypeKind::Float,
            "double" => TypeKind::Double,

            "bool" => TypeKind::Bool,
            "_Complex" => TypeKind::Complex(None),
            "_Decimal32" => TypeKind::Decimal32,
            "_Decimal64" => TypeKind::Decimal64,
            "_Decimal128" => TypeKind::Decimal128,
            t if t.starts_with("_BitInt") => {
                let mut kind = None;
                for rule in rule.into_inner() {
                    if let Rule::constant_expression = rule.as_rule() {
                        kind = Some(TypeKind::BitInt {
                            unsigned: false,
                            width_expr: Rc::clone(&self.parse_constant_expression(rule)?),
                        });
                    }
                }
                kind.unwrap()
            }
            _ => {
                let mut kind = None;
                for rule in rule.into_inner() {
                    match rule.as_rule() {
                        Rule::atomic_type_specifier => {
                            for rule in rule.into_inner() {
                                if let Rule::type_name = rule.as_rule() {
                                    match self.parse_type_name(rule)? {
                                        (r#type, type_name_decls) => {
                                            decls.extend(type_name_decls);
                                            kind = Some(TypeKind::Atomic(r#type))
                                        }
                                    }
                                }
                            }
                        }
                        Rule::typeof_specifier => {
                            let unqual = rule.as_str().starts_with("typeof_unqual");
                            for rule in rule.into_inner() {
                                match rule.as_rule() {
                                    Rule::expression => {
                                        kind = Some(TypeKind::Typeof {
                                            unqual,
                                            expr: Some(self.parse_expression(rule)?),
                                            r#type: None,
                                        })
                                    }
                                    Rule::type_name => match self.parse_type_name(rule)? {
                                        (r#type, type_name_decls) => {
                                            decls.extend(type_name_decls);
                                            kind = Some(TypeKind::Typeof {
                                                unqual,
                                                expr: None,
                                                r#type: Some(r#type),
                                            });
                                        }
                                    },
                                    _ => unreachable!(),
                                }
                            }
                        }
                        Rule::struct_or_union_specifier => {
                            match self.parse_struct_or_union_specifier(rule)? {
                                (r#type, record_decl) => {
                                    kind = Some(r#type.kind);
                                    decls.push(record_decl);
                                }
                            }
                        }
                        Rule::enum_specifier => match self.parse_enum_specifier(rule)? {
                            (r#type, enum_decl) => {
                                kind = Some(r#type.kind);
                                decls.push(enum_decl);
                            }
                        },
                        _ => unreachable!(),
                    }
                }
                kind.unwrap()
            }
        };
        Ok((
            Type {
                kind,
                ..Type::new(self.file_id, from_pest_span(span))
            },
            decls,
        ))
    }

    pub fn parse_type_qualifier(&self, rule: Pair<Rule>) -> Result<TypeQual, Diagnostic<usize>> {
        Ok(match rule.as_str() {
            "const" => TypeQual::Const,
            "restrict" => TypeQual::Restrict,
            "volatile" => TypeQual::Volatile,
            "_Atomic" => TypeQual::Atomic,
            _ => unreachable!(),
        })
    }

    pub fn parse_storage_class_specifier(
        &self,
        rule: Pair<Rule>,
    ) -> Result<StorageClass, Diagnostic<usize>> {
        let span = rule.as_span();
        Ok(StorageClass::new(
            self.file_id,
            from_pest_span(span),
            match rule.as_str() {
                "auto" => StorageClassKind::Auto,
                "constexpr" => StorageClassKind::Constexpr,
                "extern" => StorageClassKind::Extern,
                "register" => StorageClassKind::Register,
                "static" => StorageClassKind::Static,
                "thread_local" => StorageClassKind::ThreadLocal,
                "typedef" => StorageClassKind::Typedef,
                _ => unreachable!(),
            },
        ))
    }

    pub fn parse_struct_or_union_specifier(
        &self,
        rule: Pair<Rule>,
    ) -> Result<(Type, Rc<RefCell<Declaration>>), Diagnostic<usize>> {
        let record_kind = if rule.as_str().starts_with("struct") {
            RecordKind::Struct
        } else {
            RecordKind::Union
        };
        let span = rule.as_span();
        let mut name = format!("<unnamed struct {:?}>", span.start_pos().line_col());
        let mut is_declare = true;
        let mut members_decl = Vec::new();
        let mut attributes = Vec::new();
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::identifier => name = parse_identifier(rule.as_str())?,
                Rule::attribute_specifier_sequence => {
                    attributes.extend(self.parse_attribute_specifier_sequence(rule)?);
                }
                Rule::member_declaration => {
                    members_decl.extend(self.parse_member_declaration(rule)?);
                    is_declare = false;
                }
                _ => unreachable!(),
            }
        }
        let record_type = Type {
            attributes: attributes.clone(),
            kind: TypeKind::Record {
                name: name.clone(),
                kind: record_kind,
                members: None,
            },
            ..Type::new(self.file_id, from_pest_span(span))
        };
        Ok((
            record_type.clone(),
            Rc::new(RefCell::new(Declaration {
                attributes,
                name,
                //将会在语义分析的时候重新确定type
                r#type: Rc::new(RefCell::new(record_type)),
                ..Declaration::new(
                    self.file_id,
                    from_pest_span(span),
                    DeclarationKind::Record {
                        members_decl: if is_declare { None } else { Some(members_decl) },
                    },
                )
            })),
        ))
    }

    pub fn parse_enum_specifier(
        &self,
        rule: Pair<Rule>,
    ) -> Result<(Type, Rc<RefCell<Declaration>>), Diagnostic<usize>> {
        let span = rule.as_span();
        let mut name = format!("<unnamed enum {:?}>", span.start_pos().line_col());
        let mut attributes = Vec::new();
        let mut underlying_type = None;
        let mut is_declare = true;
        let mut enumerators = Vec::new();
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::identifier => name = parse_identifier(rule.as_str())?,
                Rule::attribute_specifier_sequence => {
                    attributes.extend(self.parse_attribute_specifier_sequence(rule)?);
                }
                Rule::enumerator => {
                    enumerators.push(self.parse_enumerator(rule)?);
                    is_declare = false;
                }
                Rule::specifier_qualifier_list_without_typedef
                | Rule::specifier_qualifier_list_with_typedef => {
                    match self.parse_specifier_qualifier_list(rule)? {
                        //decls一定为空
                        (mut r#type, _decls, extern_attrs) => {
                            r#type.attributes.extend(extern_attrs);
                            underlying_type = Some(r#type);
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
        let enum_type = Type {
            attributes: attributes.clone(),
            kind: TypeKind::Enum {
                name: name.clone(),
                underlying: Rc::new(RefCell::new(
                    underlying_type.unwrap_or(Type::new(self.file_id, from_pest_span(span))),
                )),
                enum_consts: None,
            },
            ..Type::new(self.file_id, from_pest_span(span))
        };
        Ok((
            enum_type.clone(),
            Rc::new(RefCell::new(Declaration {
                attributes,
                name,
                //将会在语义分析的时候重新确定type
                r#type: Rc::new(RefCell::new(enum_type)),
                ..Declaration::new(
                    self.file_id,
                    from_pest_span(span),
                    DeclarationKind::Enum {
                        enumerators: if is_declare { None } else { Some(enumerators) },
                    },
                )
            })),
        ))
    }

    pub fn parse_declarator(
        &self,
        spec_type: Type,
        rule: Pair<Rule>,
    ) -> Result<Rc<RefCell<Declaration>>, Diagnostic<usize>> {
        let span = rule.as_span();
        let mut name = String::new();
        let mut attributes = Vec::new();
        let mut final_type = spec_type.clone();
        let mut child_declarator = None;
        let mut parameter_decls = Vec::new();
        let mut function_decls = Vec::new();
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::direct_declarator | Rule::direct_abstract_declarator => {
                    for rule in rule.into_inner().rev() {
                        match rule.as_rule() {
                            Rule::name_declarator => {
                                for rule in rule.into_inner() {
                                    match rule.as_rule() {
                                        Rule::identifier => name = parse_identifier(rule.as_str())?,
                                        Rule::attribute_specifier_sequence => {
                                            attributes.extend(
                                                self.parse_attribute_specifier_sequence(rule)?,
                                            );
                                        }
                                        _ => unreachable!(),
                                    }
                                }
                            }
                            Rule::declarator | Rule::abstract_declarator => {
                                child_declarator = Some(rule)
                            }
                            Rule::function_declarator | Rule::function_abstract_declarator => {
                                //避免冲突
                                parameter_decls.clear();

                                let span = rule.as_span();
                                let mut attributes = Vec::new();
                                let has_varparam = rule.as_str().contains("...");
                                let mut parameters_type = Vec::new();
                                for rule in rule.into_inner() {
                                    match rule.as_rule() {
                                        Rule::parameter_declaration => parameter_decls
                                            .extend(self.parse_parameter_declaration(rule)?),
                                        Rule::attribute_specifier_sequence => {
                                            attributes.extend(
                                                self.parse_attribute_specifier_sequence(rule)?,
                                            );
                                        }
                                        _ => unreachable!(),
                                    }
                                }
                                for decl in &parameter_decls {
                                    if let DeclarationKind::Parameter = &decl.borrow().kind {
                                        parameters_type.push(decl.borrow().r#type.clone());
                                    }
                                }
                                final_type = Type {
                                    attributes,
                                    kind: TypeKind::Function {
                                        return_type: Rc::new(RefCell::new(final_type)),
                                        parameters_type,
                                        has_varparam,
                                    },
                                    ..Type::new(self.file_id, from_pest_span(span))
                                };

                                //需要额外保存函数声明, 以便之后对其进行类型检查
                                function_decls.push(Rc::new(RefCell::new(Declaration {
                                    r#type: Rc::new(RefCell::new(final_type.clone())),
                                    ..Declaration::new(
                                        self.file_id,
                                        from_pest_span(span),
                                        DeclarationKind::Function {
                                            parameter_decls: parameter_decls.clone(),
                                            function_specs: vec![],
                                            body: None,
                                            symtab: None,
                                        },
                                    )
                                })));
                            }
                            Rule::array_declarator | Rule::array_abstract_declarator => {
                                let span = rule.as_span();
                                let mut attributes = Vec::new();
                                let mut has_static = false;
                                let mut has_star = false;
                                let mut len_expr = None;
                                let mut type_quals = Vec::new();

                                for rule in rule.into_inner() {
                                    match rule.as_rule() {
                                        Rule::type_qualifier_list => {
                                            type_quals.extend(self.parse_type_qualifier_list(rule)?)
                                        }
                                        Rule::assignment_expression => {
                                            len_expr = Some(self.parse_assignment_expression(rule)?)
                                        }
                                        Rule::attribute_specifier_sequence => attributes
                                            .extend(self.parse_attribute_specifier_sequence(rule)?),
                                        Rule::star => has_star = true,
                                        Rule::r#static => has_static = true,
                                        _ => unreachable!(),
                                    }
                                }

                                final_type = Type {
                                    kind: TypeKind::Array {
                                        has_static,
                                        has_star,
                                        element_type: Rc::new(RefCell::new(final_type)),
                                        len_expr,
                                    },
                                    ..Type::new(self.file_id, from_pest_span(span))
                                };
                                if type_quals.len() > 0 {
                                    final_type = Type {
                                        kind: TypeKind::Qualified {
                                            qualifiers: type_quals,
                                            r#type: Rc::new(RefCell::new(final_type)),
                                        },
                                        ..Type::new(self.file_id, from_pest_span(span))
                                    };
                                }
                                final_type.attributes = attributes;
                            }
                            _ => unreachable!(),
                        }
                    }
                }
                Rule::pointer => {
                    let span = rule.as_span();
                    let mut attributes = Vec::new();
                    let mut type_quals = Vec::new();

                    for rule in rule.into_inner() {
                        match rule.as_rule() {
                            Rule::attribute_specifier_sequence => {
                                attributes.extend(self.parse_attribute_specifier_sequence(rule)?);
                            }
                            Rule::type_qualifier_list => {
                                type_quals.extend(self.parse_type_qualifier_list(rule)?)
                            }
                            _ => unreachable!(),
                        }
                    }
                    final_type = Type {
                        kind: TypeKind::Pointer(Rc::new(RefCell::new(final_type))),
                        ..Type::new(self.file_id, from_pest_span(span))
                    };
                    if type_quals.len() > 0 {
                        final_type = Type {
                            kind: TypeKind::Qualified {
                                qualifiers: type_quals,
                                r#type: Rc::new(RefCell::new(final_type)),
                            },
                            ..Type::new(self.file_id, from_pest_span(span))
                        };
                    }
                    final_type.attributes = attributes;
                }
                _ => unreachable!(),
            }
        }
        if let Some(rule) = child_declarator {
            let parent = self.parse_declarator(final_type, rule)?;
            parent.borrow_mut().children.extend(function_decls);
            Ok(parent)
        } else {
            let kind = match &final_type.kind {
                TypeKind::Function { .. } => {
                    function_decls.pop(); //此时最后一个元素一定与接下来要返回的Declaration相同
                    DeclarationKind::Function {
                        parameter_decls,
                        function_specs: Vec::new(),
                        body: None,
                        symtab: None,
                    }
                }
                _ => DeclarationKind::Var { initializer: None },
            };

            Ok(Rc::new(RefCell::new(Declaration {
                attributes,
                name,
                r#type: Rc::new(RefCell::new(final_type.clone())),
                children: function_decls,
                ..Declaration::new(self.file_id, from_pest_span(span), kind)
            })))
        }
    }

    pub fn parse_member_declaration(
        &self,
        rule: Pair<Rule>,
    ) -> Result<Vec<Rc<RefCell<Declaration>>>, Diagnostic<usize>> {
        let span = rule.as_span();
        let mut decls = Vec::new();
        let mut attributes = Vec::new();
        let mut type_attrs = Vec::new();
        let mut spec_type = None;
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::attribute_specifier_sequence => {
                    attributes.extend(self.parse_attribute_specifier_sequence(rule)?);
                }
                Rule::specifier_qualifier_list_without_typedef
                | Rule::specifier_qualifier_list_with_typedef => {
                    match self.parse_specifier_qualifier_list(rule)? {
                        (r#type, spec_decls, extern_attrs) => {
                            type_attrs.extend(extern_attrs);
                            spec_type = Some(r#type);
                            decls.extend(spec_decls);
                        }
                    }
                }
                Rule::member_declarator => {
                    let decl = self.parse_member_declarator(spec_type.clone().unwrap(), rule)?;
                    decl.borrow_mut()
                        .r#type
                        .borrow_mut()
                        .attributes
                        .extend(type_attrs.clone());
                    decl.borrow_mut().attributes.extend(attributes.clone());
                    decls.push(decl);
                }
                Rule::static_assert_declaration => {
                    decls.push(self.parse_static_assert_declaration(rule)?);
                }
                _ => unreachable!(),
            }
        }
        if let Some(mut spec_type) = spec_type
            && decls.len() == 0
        {
            //没有declarator时的情况
            decls.push(Rc::new(RefCell::new(Declaration {
                attributes,
                r#type: Rc::new(RefCell::new({
                    spec_type.attributes.extend(type_attrs);
                    spec_type
                })),
                ..Declaration::new(
                    self.file_id,
                    from_pest_span(span),
                    DeclarationKind::Member { bit_field: None },
                )
            })));
        }
        Ok(self.filter_decls(decls))
    }

    pub fn parse_member_declarator(
        &self,
        spec_type: Type,
        rule: Pair<Rule>,
    ) -> Result<Rc<RefCell<Declaration>>, Diagnostic<usize>> {
        let span = rule.as_span();
        let mut decl = None;
        let mut bit_field = None;
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::declarator => decl = Some(self.parse_declarator(spec_type.clone(), rule)?),
                Rule::constant_expression => {
                    bit_field = Some(self.parse_constant_expression(rule)?)
                }
                _ => unreachable!(),
            }
        }
        if let Some(t) = decl {
            {
                let Declaration {
                    span: part_span,
                    kind,
                    ..
                } = &mut *t.borrow_mut();
                *part_span = from_pest_span(span);
                *kind = DeclarationKind::Member { bit_field };
            }
            Ok(t)
        } else {
            Ok(Rc::new(RefCell::new(Declaration {
                r#type: Rc::new(RefCell::new(spec_type)),
                ..Declaration::new(
                    self.file_id,
                    from_pest_span(span),
                    DeclarationKind::Member { bit_field },
                )
            })))
        }
    }

    pub fn parse_specifier_qualifier_list(
        &self,
        rule: Pair<Rule>,
    ) -> Result<
        (
            Type,
            Vec<Rc<RefCell<Declaration>>>,
            Vec<Rc<RefCell<Attribute>>>,
        ),
        Diagnostic<usize>,
    > {
        //两者结构相同, 但specifier_qualifier_list内容要少一点
        Ok(match self.parse_declaration_specifiers(rule)? {
            (
                r#type,
                _storage_classes,
                _function_specs,
                decls,
                extern_type_attrs,
                _extern_var_attrs,
            ) => (r#type, decls, extern_type_attrs),
        })
    }

    pub fn parse_enumerator(
        &self,
        rule: Pair<Rule>,
    ) -> Result<Rc<RefCell<Declaration>>, Diagnostic<usize>> {
        let span = rule.as_span();
        let mut name = String::new();
        let mut expr = None;
        let mut attributes = Vec::new();
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::identifier => name = parse_identifier(rule.as_str())?,
                Rule::attribute_specifier_sequence => {
                    attributes.extend(self.parse_attribute_specifier_sequence(rule)?)
                }
                Rule::constant_expression => expr = Some(self.parse_constant_expression(rule)?),
                _ => unreachable!(),
            }
        }
        Ok(Rc::new(RefCell::new(Declaration {
            attributes,
            name,
            ..Declaration::new(
                self.file_id,
                from_pest_span(span),
                DeclarationKind::Enumerator { value: expr },
            )
        })))
    }

    pub fn parse_parameter_declaration(
        &self,
        rule: Pair<Rule>,
    ) -> Result<Vec<Rc<RefCell<Declaration>>>, Diagnostic<usize>> {
        let mut decls = Vec::new();
        //两者结构相同
        for decl in self.parse_declaration(rule)? {
            match &mut *decl.borrow_mut() {
                Declaration {
                    kind: kind @ DeclarationKind::Var { .. },
                    ..
                } => {
                    *kind = DeclarationKind::Parameter;
                }
                _ => {}
            }
            decls.push(decl);
        }
        Ok(self.filter_decls(decls))
    }

    pub fn parse_type_qualifier_list(
        &self,
        rule: Pair<Rule>,
    ) -> Result<Vec<TypeQual>, Diagnostic<usize>> {
        let mut type_quals = Vec::new();
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::type_qualifier => type_quals.push(self.parse_type_qualifier(rule)?),
                _ => unreachable!(),
            }
        }
        Ok(type_quals)
    }

    pub fn parse_type_name(
        &self,
        rule: Pair<Rule>,
    ) -> Result<(Rc<RefCell<Type>>, Vec<Rc<RefCell<Declaration>>>), Diagnostic<usize>> {
        let mut spec_type = None;
        let mut type_attrs = Vec::new();
        let mut r#type = None;
        let mut decls = Vec::new();
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::specifier_qualifier_list_without_typedef
                | Rule::specifier_qualifier_list_with_typedef => {
                    match self.parse_specifier_qualifier_list(rule)? {
                        (r#type, spec_decls, extern_attrs) => {
                            spec_type = Some(r#type);
                            type_attrs.extend(extern_attrs);
                            decls.extend(spec_decls);
                        }
                    }
                }
                Rule::abstract_declarator => {
                    let decl = self.parse_declarator(spec_type.clone().unwrap(), rule)?;
                    decl.borrow_mut()
                        .r#type
                        .borrow_mut()
                        .attributes
                        .extend(type_attrs.clone());
                    r#type = Some(decl.borrow().r#type.clone());
                }
                _ => unreachable!(),
            }
        }
        Ok((
            match r#type {
                Some(t) => t,
                None => {
                    let mut spec_type = spec_type.unwrap();
                    spec_type.attributes.extend(type_attrs);
                    Rc::new(RefCell::new(spec_type))
                }
            },
            decls,
        ))
    }

    pub fn parse_static_assert_declaration(
        &self,
        rule: Pair<Rule>,
    ) -> Result<Rc<RefCell<Declaration>>, Diagnostic<usize>> {
        let span = rule.as_span();
        let mut expr = None;
        let mut message = String::new();
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::constant_expression => expr = Some(self.parse_constant_expression(rule)?),
                Rule::string_literal => {
                    message = if let ExprKind::String { prefix: _, text } =
                        &self.parse_string_literal(rule)?.borrow().kind
                    {
                        text.clone()
                    } else {
                        unreachable!()
                    }
                }
                _ => unreachable!(),
            }
        }
        Ok(Rc::new(RefCell::new(Declaration {
            name: message,
            ..Declaration::new(
                self.file_id,
                from_pest_span(span),
                DeclarationKind::StaticAssert {
                    expr: expr.unwrap(),
                },
            )
        })))
    }
}
