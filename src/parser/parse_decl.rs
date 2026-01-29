use super::{CParser, Rule};
use crate::ast::decl::{
    Declaration, DeclarationKind, FunctionSpec, FunctionSpecKind, StorageClass, StorageClassKind,
};
use crate::ast::expr::ExprKind;
use crate::ast::{Attribute, AttributeKind};
use crate::ctype::TypeQual;
use crate::ctype::{RecordKind, Type, TypeKind};
use pest::error::ErrorVariant;
use pest::{error::Error, iterators::Pair};
use std::cell::RefCell;
use std::rc::Rc;

impl<'a> CParser<'a> {
    pub fn parse_function_definition(
        &self,
        rule: Pair<'a, Rule>,
    ) -> Result<Vec<Rc<RefCell<Declaration<'a>>>>, Error<Rule>> {
        let mut decls = Vec::new();
        let mut attributes = Vec::new();
        let mut spec_type = None;
        let mut storage_classes = Vec::new();
        let mut function_specs = Vec::new();
        let mut function_decl = None;
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::attribute_specifier_sequence => {
                    attributes.extend(self.parse_attribute_specifier_sequence(rule)?);
                }
                Rule::declaration_specifiers => {
                    let mut parse_result = self.parse_declaration_specifiers(rule)?;
                    if let Some(t) = parse_result.0.attributes.iter().position(|x| {
                        if let AttributeKind::AlignAs { r#type: _, expr: _ } = &x.borrow().kind {
                            true
                        } else {
                            false
                        }
                    }) {
                        attributes.push(parse_result.0.attributes.remove(t));
                    }
                    spec_type = Some(parse_result.0);
                    storage_classes = parse_result.1;
                    function_specs = parse_result.2;
                    let spec_decl = parse_result.3;
                    decls.extend(spec_decl);
                }
                Rule::declarator => {
                    let span = rule.as_span();
                    let decl = self.parse_declarator(spec_type.clone().unwrap(), rule)?;
                    decl.borrow_mut().attributes.extend(attributes.clone());
                    decl.borrow_mut()
                        .storage_classes
                        .extend(storage_classes.clone());
                    if let DeclarationKind::Function {
                        parameter_decls: _,
                        function_specs: t,
                        body: _,
                    } = &mut decl.borrow_mut().kind
                    {
                        t.extend(function_specs.clone());
                    } else {
                        return Err(Error::new_from_span(
                            ErrorVariant::CustomError {
                                message: format!("not a function"),
                            },
                            span,
                        ));
                    }
                    function_decl = Some(decl);
                }
                Rule::compound_statement => {
                    if let Some(function_decl) = &function_decl {
                        if let DeclarationKind::Function {
                            parameter_decls: _,
                            function_specs: _,
                            body,
                        } = &mut function_decl.borrow_mut().kind
                        {
                            *body = Some(self.parse_compound_statement(rule)?);
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
        decls.push(function_decl.unwrap());
        Ok(decls)
    }

    pub fn parse_declaration(
        &self,
        rule: Pair<'a, Rule>,
    ) -> Result<Vec<Rc<RefCell<Declaration<'a>>>>, Error<Rule>> {
        let span = rule.as_span();
        let mut decls = Vec::new();
        let mut attributes = Vec::new();
        let mut spec_type = None;
        let mut storage_classes = Vec::new();
        let mut function_specs = Vec::new();
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::attribute_specifier_sequence => {
                    attributes.extend(self.parse_attribute_specifier_sequence(rule)?);
                }
                Rule::declaration_specifiers => {
                    let mut parse_result = self.parse_declaration_specifiers(rule)?;
                    if let Some(t) = parse_result.0.attributes.iter().position(|x| {
                        if let AttributeKind::AlignAs { r#type: _, expr: _ } = &x.borrow().kind {
                            true
                        } else {
                            false
                        }
                    }) {
                        attributes.push(parse_result.0.attributes.remove(t));
                    }
                    spec_type = Some(parse_result.0);
                    storage_classes = parse_result.1;
                    function_specs = parse_result.2;
                    let spec_decl = parse_result.3;
                    decls.extend(spec_decl);
                }
                Rule::initializer => {
                    let initializer = self.parse_initializer(rule)?;
                    let decl = decls.last_mut().unwrap();
                    let mut decl_borrow = decl.borrow_mut();
                    if let DeclarationKind::Var { initializer: old } = &mut decl_borrow.kind {
                        *old = Some(initializer);
                    } else {
                        return Err(Error::new_from_span(
                            ErrorVariant::CustomError {
                                message: format!("only variables can be initialized"),
                            },
                            initializer.borrow().span,
                        ));
                    }
                    decl_borrow.attributes.extend(attributes.clone());
                }
                Rule::declarator => {
                    let decl = self.parse_declarator(spec_type.clone().unwrap(), rule)?;
                    decl.borrow_mut().attributes.extend(attributes.clone());
                    if storage_classes
                        .iter()
                        .map(|x| &x.kind)
                        .collect::<Vec<&StorageClassKind>>()
                        .contains(&&StorageClassKind::Typedef)
                    {
                        decl.borrow_mut().kind = DeclarationKind::Type;
                    }
                    if let DeclarationKind::Function {
                        parameter_decls: _,
                        function_specs: t,
                        body: _,
                    } = &mut decl.borrow_mut().kind
                    {
                        t.extend(function_specs.clone());
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
                        span,
                        attributes: attributes.clone(),
                        name: String::new(),
                        r#type: Rc::new(RefCell::new(Type {
                            span,
                            attributes: attributes.clone(),
                            kind: TypeKind::Void,
                        })),
                        storage_classes: storage_classes.clone(),
                        kind: DeclarationKind::Attribute,
                    })))
                }
                _ => unreachable!(),
            }
        }
        if let Some(spec_type) = spec_type
            && decls.len() == 0
        {
            //没有declarator时的情况
            //主要是针对abstract declaration的
            decls.push(Rc::new(RefCell::new(Declaration {
                span,
                attributes,
                name: String::new(),
                r#type: Rc::new(RefCell::new(spec_type)),
                storage_classes,
                kind: DeclarationKind::Var { initializer: None },
            })));
        }
        Ok(decls)
    }

    pub fn parse_declaration_specifiers(
        &self,
        rule: Pair<'a, Rule>,
    ) -> Result<
        (
            //因为解析出来的不是最终的类型, 所以不需要引用计数
            Type<'a>,
            Vec<StorageClass<'a>>,
            Vec<FunctionSpec<'a>>,
            //可能有record或enum的定义
            Vec<Rc<RefCell<Declaration<'a>>>>,
        ),
        Error<Rule>,
    > {
        let span = rule.as_span();
        let mut attributes = Vec::new();
        let mut types = Vec::new();
        let mut qualifiers = Vec::new();
        let mut storage_classes = Vec::new();
        let mut function_specs = Vec::new();
        let mut decls = Vec::new();
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::type_qualifier => qualifiers.push(self.parse_type_qualifier(rule)?),
                Rule::storage_class_specifier => match self.parse_storage_class_specifier(rule)? {
                    StorageClass { kind, span } if matches!(kind, StorageClassKind::Auto) => {
                        types.push(Type {
                            span,
                            attributes: vec![],
                            kind: TypeKind::Auto(None),
                        });
                    }
                    t => storage_classes.push(t),
                },
                Rule::function_specifier => function_specs.push(FunctionSpec {
                    span: rule.as_span(),
                    kind: match rule.as_str() {
                        "inline" => FunctionSpecKind::Inline,
                        "_Noreturn" => FunctionSpecKind::Noreturn,
                        _ => unreachable!(),
                    },
                }),
                Rule::type_specifier => {
                    let parse_result = self.parse_type_specifier(rule)?;
                    types.push(parse_result.0);
                    decls.extend(parse_result.1);
                }
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
                            Rule::type_name => {
                                r#type = Some(self.parse_type_name(rule)?);
                            }
                            _ => unreachable!(),
                        }
                    }
                    attributes.push(Rc::new(RefCell::new(Attribute {
                        span,
                        prefix_name: None,
                        name: "alignas".to_string(),
                        kind: AttributeKind::AlignAs { r#type, expr },
                    })));
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
            return Err(Error::new_from_span(
                ErrorVariant::CustomError {
                    message: format!("type specifiers missing"),
                },
                span,
            ));
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
                                span: x.span,
                                attributes: x.attributes.clone(),
                                kind: TypeKind::LongDouble,
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
            Err(Error::new_from_span(
                ErrorVariant::CustomError {
                    message: format!("cannot combine {} and {}", acc, x),
                },
                x.span,
            ))
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
                span,
                attributes: attributes.clone(),
                kind: TypeKind::Atomic(Rc::new(RefCell::new(final_type))),
            };
            qualifiers.remove(atomic_index);
        }
        if qualifiers.len() > 0 {
            final_type = Type {
                span,
                attributes,
                kind: TypeKind::Qualified {
                    qualifiers,
                    r#type: Rc::new(RefCell::new(final_type)),
                },
            };
        } else {
            final_type.span = span;
            final_type.attributes.extend(attributes);
        }
        Ok((final_type, storage_classes, function_specs, decls))
    }

    pub fn parse_type_specifier(
        &self,
        rule: Pair<'a, Rule>,
    ) -> Result<(Type<'a>, Vec<Rc<RefCell<Declaration<'a>>>>), Error<Rule>> {
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
            "_BitInt" => {
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
            "bool" => TypeKind::Bool,
            "_Complex" => TypeKind::Complex(None),
            "_Decimal32" => TypeKind::Decimal32,
            "_Decimal64" => TypeKind::Decimal64,
            "_Decimal128" => TypeKind::Decimal128,
            _ => {
                let mut kind = None;
                for rule in rule.into_inner() {
                    match rule.as_rule() {
                        Rule::atomic_type_specifier => {
                            for rule in rule.into_inner() {
                                if let Rule::type_name = rule.as_rule() {
                                    kind = Some(TypeKind::Atomic(self.parse_type_name(rule)?));
                                }
                            }
                        }
                        Rule::typedef_name => {
                            for rule in rule.into_inner() {
                                if let Rule::identifier = rule.as_rule() {
                                    kind = Some(TypeKind::Typedef {
                                        name: rule.as_str().to_string(),
                                        r#type: None,
                                    })
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
                                    Rule::type_name => {
                                        kind = Some(TypeKind::Typeof {
                                            unqual,
                                            expr: None,
                                            r#type: Some(self.parse_type_name(rule)?),
                                        });
                                    }
                                    _ => unreachable!(),
                                }
                            }
                        }
                        Rule::struct_or_union_specifier => {
                            let parse_result = self.parse_struct_or_union_specifier(rule)?;
                            kind = Some(parse_result.0.kind);
                            if let Some(t) = parse_result.1 {
                                decls.push(t);
                            }
                        }
                        Rule::enum_specifier => {
                            let parse_result = self.parse_enum_specifier(rule)?;
                            kind = Some(parse_result.0.kind);
                            if let Some(t) = parse_result.1 {
                                decls.push(t);
                            }
                        }
                        _ => unreachable!(),
                    }
                }
                kind.unwrap()
            }
        };
        Ok((
            Type {
                span,
                attributes: Vec::new(),
                kind: kind,
            },
            decls,
        ))
    }

    pub fn parse_type_qualifier(&self, rule: Pair<'a, Rule>) -> Result<TypeQual, Error<Rule>> {
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
        rule: Pair<'a, Rule>,
    ) -> Result<StorageClass<'a>, Error<Rule>> {
        Ok(StorageClass {
            span: rule.as_span(),
            kind: match rule.as_str() {
                "auto" => StorageClassKind::Auto,
                "constexpr" => StorageClassKind::Constexpr,
                "extern" => StorageClassKind::Extern,
                "register" => StorageClassKind::Register,
                "static" => StorageClassKind::Static,
                "thread_local" => StorageClassKind::ThreadLocal,
                "typedef" => StorageClassKind::Typedef,
                _ => unreachable!(),
            },
        })
    }

    pub fn parse_struct_or_union_specifier(
        &self,
        rule: Pair<'a, Rule>,
    ) -> Result<(Type<'a>, Option<Rc<RefCell<Declaration<'a>>>>), Error<Rule>> {
        let record_kind = if rule.as_str().starts_with("struct") {
            RecordKind::Struct
        } else {
            RecordKind::Union
        };
        let span = rule.as_span();
        let mut name = format!("<unnamed struct {:?}>", span.start_pos().line_col());
        let mut members_decl = Vec::new();
        let mut attributes = Vec::new();
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::identifier => name = rule.as_str().to_string(),
                Rule::attribute_specifier_sequence => {
                    attributes.extend(self.parse_attribute_specifier_sequence(rule)?);
                }
                Rule::member_declaration => {
                    members_decl.extend(self.parse_member_declaration(rule)?);
                }
                _ => unreachable!(),
            }
        }
        let record_type = Type {
            span,
            attributes: attributes.clone(),
            kind: TypeKind::Record {
                name: name.clone(),
                kind: record_kind,
                members: None,
            },
        };
        Ok((
            record_type.clone(),
            Some(Rc::new(RefCell::new(Declaration {
                span,
                attributes: attributes.clone(),
                name: name,
                //将会在语义分析的时候重新确定type
                r#type: Rc::new(RefCell::new(record_type)),
                storage_classes: Vec::new(),
                kind: DeclarationKind::Record {
                    members_decl: if members_decl.len() > 0 {
                        Some(members_decl)
                    } else {
                        None
                    },
                },
            }))),
        ))
    }

    pub fn parse_enum_specifier(
        &self,
        rule: Pair<'a, Rule>,
    ) -> Result<(Type<'a>, Option<Rc<RefCell<Declaration<'a>>>>), Error<Rule>> {
        let span = rule.as_span();
        let mut name = format!("<unnamed enum {:?}>", span.start_pos().line_col());
        let mut attributes = Vec::new();
        let mut underlying_type = None;
        let mut enumerators = Vec::new();
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::identifier => name = rule.as_str().to_string(),
                Rule::attribute_specifier_sequence => {
                    attributes.extend(self.parse_attribute_specifier_sequence(rule)?);
                }
                Rule::enumerator => enumerators.push(self.parse_enumerator(rule)?),
                Rule::specifier_qualifier_list => {
                    underlying_type = Some(self.parse_specifier_qualifier_list(rule)?.0);
                }
                _ => unreachable!(),
            }
        }
        let enum_type = Type {
            span,
            attributes: attributes.clone(),
            kind: TypeKind::Enum {
                name: name.clone(),
                underlying: Rc::new(RefCell::new(underlying_type.unwrap_or(Type {
                    span,
                    attributes: Vec::new(),
                    kind: TypeKind::Error, //underlying type将在类型检查时确定
                }))),
                enum_consts: None,
            },
        };
        Ok((
            enum_type.clone(),
            Some(Rc::new(RefCell::new(Declaration {
                span,
                attributes: attributes.clone(),
                name,
                //将会在语义分析的时候重新确定type
                r#type: Rc::new(RefCell::new(enum_type)),
                storage_classes: Vec::new(),
                kind: DeclarationKind::Enum {
                    enumerators: if enumerators.len() > 0 {
                        Some(enumerators)
                    } else {
                        None
                    },
                },
            }))),
        ))
    }

    pub fn parse_declarator(
        &self,
        spec_type: Type<'a>,
        rule: Pair<'a, Rule>,
    ) -> Result<Rc<RefCell<Declaration<'a>>>, Error<Rule>> {
        let span = rule.as_span();
        let mut name = String::new();
        let mut attributes = Vec::new();
        let mut final_type = spec_type.clone();
        let mut child_declarator = None;
        let mut parameter_decls = Vec::new();
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::direct_declarator | Rule::direct_abstract_declarator => {
                    for rule in rule.into_inner().rev() {
                        match rule.as_rule() {
                            Rule::name_declarator => {
                                for rule in rule.into_inner() {
                                    match rule.as_rule() {
                                        Rule::identifier => name = rule.as_str().to_string(),
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
                                let span = rule.as_span();
                                let mut attributes = Vec::new();
                                let has_varparam = rule.as_str().contains("...");
                                let mut parameters_type = Vec::new();
                                for rule in rule.into_inner() {
                                    match rule.as_rule() {
                                        Rule::parameter_declaration => {
                                            //因为不能返回函数类型, 所以这里不会冲突
                                            parameter_decls
                                                .extend(self.parse_parameter_declaration(rule)?)
                                        }
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
                                    span,
                                    attributes,
                                    kind: TypeKind::Function {
                                        return_type: Rc::new(RefCell::new(final_type)),
                                        parameters_type,
                                        has_varparam,
                                    },
                                }
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
                                    span,
                                    attributes: Vec::new(),
                                    kind: TypeKind::Array {
                                        has_static,
                                        has_star,
                                        element_type: Rc::new(RefCell::new(final_type)),
                                        len_expr,
                                    },
                                };
                                if type_quals.len() > 0 {
                                    final_type = Type {
                                        span,
                                        attributes: Vec::new(),
                                        kind: TypeKind::Qualified {
                                            qualifiers: type_quals,
                                            r#type: Rc::new(RefCell::new(final_type)),
                                        },
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
                        span,
                        attributes: Vec::new(),
                        kind: TypeKind::Pointer(Rc::new(RefCell::new(final_type))),
                    };
                    if type_quals.len() > 0 {
                        final_type = Type {
                            span,
                            attributes: Vec::new(),
                            kind: TypeKind::Qualified {
                                qualifiers: type_quals,
                                r#type: Rc::new(RefCell::new(final_type)),
                            },
                        };
                    }
                    final_type.attributes = attributes;
                }
                _ => unreachable!(),
            }
        }
        if let Some(rule) = child_declarator {
            //这么做可能会忽略当前的parameter_decls
            return self.parse_declarator(final_type, rule);
        }
        let kind = match &final_type.kind {
            TypeKind::Function { .. } => DeclarationKind::Function {
                parameter_decls,
                function_specs: Vec::new(),
                body: None,
            },
            _ => DeclarationKind::Var { initializer: None },
        };
        Ok(Rc::new(RefCell::new(Declaration {
            span,
            attributes,
            name,
            r#type: Rc::new(RefCell::new(final_type)),
            storage_classes: Vec::new(),
            kind,
        })))
    }

    pub fn parse_member_declaration(
        &self,
        rule: Pair<'a, Rule>,
    ) -> Result<Vec<Rc<RefCell<Declaration<'a>>>>, Error<Rule>> {
        let mut decls = Vec::new();
        let mut attributes = Vec::new();
        let mut spec_type = None;
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::attribute_specifier_sequence => {
                    attributes.extend(self.parse_attribute_specifier_sequence(rule)?);
                }
                Rule::specifier_qualifier_list => {
                    let mut parse_result = self.parse_specifier_qualifier_list(rule)?;
                    if let Some(t) = parse_result.0.attributes.iter().position(|x| {
                        if let AttributeKind::AlignAs { r#type: _, expr: _ } = &x.borrow().kind {
                            true
                        } else {
                            false
                        }
                    }) {
                        attributes.push(parse_result.0.attributes.remove(t));
                    }
                    spec_type = Some(parse_result.0);
                    let spec_decl = parse_result.1;
                    decls.extend(spec_decl);
                }
                Rule::member_declarator => {
                    let decl = self.parse_member_declarator(spec_type.clone().unwrap(), rule)?;
                    decl.borrow_mut().attributes.extend(attributes.clone());
                    decls.push(decl);
                }
                Rule::static_assert_declaration => {
                    decls.push(self.parse_static_assert_declaration(rule)?);
                }
                _ => unreachable!(),
            }
        }
        Ok(decls)
    }

    pub fn parse_member_declarator(
        &self,
        spec_type: Type<'a>,
        rule: Pair<'a, Rule>,
    ) -> Result<Rc<RefCell<Declaration<'a>>>, Error<Rule>> {
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
                    attributes: _,
                    name: _,
                    r#type: _,
                    storage_classes: _,
                    kind,
                } = &mut *t.borrow_mut();
                *part_span = span;
                *kind = DeclarationKind::Member { bit_field };
            }
            Ok(t)
        } else {
            Ok(Rc::new(RefCell::new(Declaration {
                span,
                attributes: Vec::new(),
                name: String::new(),
                r#type: Rc::new(RefCell::new(spec_type)),
                storage_classes: Vec::new(),
                kind: DeclarationKind::Member { bit_field },
            })))
        }
    }

    pub fn parse_specifier_qualifier_list(
        &self,
        rule: Pair<'a, Rule>,
    ) -> Result<(Type<'a>, Vec<Rc<RefCell<Declaration<'a>>>>), Error<Rule>> {
        //两者结构相同
        let parse_result = self.parse_declaration_specifiers(rule)?;
        Ok((parse_result.0, parse_result.3))
    }

    pub fn parse_enumerator(
        &self,
        rule: Pair<'a, Rule>,
    ) -> Result<Rc<RefCell<Declaration<'a>>>, Error<Rule>> {
        let span = rule.as_span();
        let mut name = String::new();
        let mut expr = None;
        let mut attributes = Vec::new();
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::identifier => name = rule.as_str().to_string(),
                Rule::attribute_specifier_sequence => {
                    attributes.extend(self.parse_attribute_specifier_sequence(rule)?)
                }
                Rule::constant_expression => expr = Some(self.parse_constant_expression(rule)?),
                _ => unreachable!(),
            }
        }
        Ok(Rc::new(RefCell::new(Declaration {
            span,
            attributes,
            name,
            r#type: Rc::new(RefCell::new(Type {
                span,
                attributes: Vec::new(),
                kind: TypeKind::Void, //真正的类型在类型检查时确定
            })),
            storage_classes: Vec::new(),
            kind: DeclarationKind::Enumerator { value: expr },
        })))
    }

    pub fn parse_parameter_declaration(
        &self,
        rule: Pair<'a, Rule>,
    ) -> Result<Vec<Rc<RefCell<Declaration<'a>>>>, Error<Rule>> {
        let mut decls = Vec::new();
        //两者结构相同
        for decl in self.parse_declaration(rule)? {
            {
                let Declaration {
                    span: _,
                    attributes: _,
                    name: _,
                    r#type: _,
                    storage_classes: _,
                    kind,
                } = &mut *decl.borrow_mut();
                match kind {
                    DeclarationKind::Var { initializer: _ } => *kind = DeclarationKind::Parameter,
                    _ => {}
                }
            }
            decls.push(decl);
        }
        Ok(decls)
    }

    pub fn parse_type_qualifier_list(
        &self,
        rule: Pair<'a, Rule>,
    ) -> Result<Vec<TypeQual>, Error<Rule>> {
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
        rule: Pair<'a, Rule>,
    ) -> Result<Rc<RefCell<Type<'a>>>, Error<Rule>> {
        let mut spec_type = None;
        let mut attributes = Vec::new();
        let mut r#type = None;
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::specifier_qualifier_list => {
                    let mut parse_result = self.parse_specifier_qualifier_list(rule)?;
                    if let Some(t) = parse_result.0.attributes.iter().position(|x| {
                        if let AttributeKind::AlignAs { r#type: _, expr: _ } = &x.borrow().kind {
                            true
                        } else {
                            false
                        }
                    }) {
                        attributes.push(parse_result.0.attributes.remove(t));
                    }
                    spec_type = Some(parse_result.0);
                }
                Rule::abstract_declarator => {
                    let decl = self.parse_declarator(spec_type.clone().unwrap(), rule)?;
                    decl.borrow_mut().attributes.extend(attributes.clone());
                    r#type = Some(decl.borrow().r#type.clone());
                }
                _ => unreachable!(),
            }
        }
        Ok(match r#type {
            Some(t) => t,
            None => Rc::new(RefCell::new(spec_type.unwrap())),
        })
    }

    pub fn parse_static_assert_declaration(
        &self,
        rule: Pair<'a, Rule>,
    ) -> Result<Rc<RefCell<Declaration<'a>>>, Error<Rule>> {
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
            span,
            attributes: Vec::new(),
            name: message,
            r#type: Rc::new(RefCell::new(Type {
                span,
                attributes: Vec::new(),
                kind: TypeKind::Void,
            })),
            storage_classes: Vec::new(),
            kind: DeclarationKind::StaticAssert {
                expr: expr.unwrap(),
            },
        })))
    }
}
