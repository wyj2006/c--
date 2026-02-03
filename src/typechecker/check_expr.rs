use super::TypeChecker;
use crate::{
    ast::{
        AttributeKind, InitializerKind,
        decl::StorageClassKind,
        expr::{BinOpKind, EncodePrefix, Expr, ExprKind, UnaryOpKind},
    },
    ctype::{
        Type, TypeKind, TypeQual, arith_result_type, array_element,
        cast::{
            array_to_ptr, func_to_ptr, integer_promote, lvalue_cast, remove_qualifier,
            usual_arith_cast,
        },
        is_compatible, pointee,
    },
    diagnostic::warning,
    symtab::{Namespace, SymbolKind},
    typechecker::Context,
    variant::Variant,
};
use codespan_reporting::diagnostic::{Diagnostic, Label, LabelStyle};
use num::BigUint;
use num::{BigInt, BigRational, FromPrimitive, Num};
use std::{cell::RefCell, rc::Rc, str::FromStr};

impl TypeChecker {
    pub fn wrap_implicit_cast(&self, expr: Rc<RefCell<Expr>>, r#type: Rc<RefCell<Type>>) -> Expr {
        Expr {
            r#type,
            value: expr.borrow().value.clone(),
            ..Expr::new(
                expr.borrow().file_id,
                expr.borrow().span,
                ExprKind::Cast {
                    is_implicit: true,
                    target: Rc::clone(&expr),
                    decls: Vec::new(),
                },
            )
        }
    }

    //尝试进行隐式转换, 失败返回None, 成功返回修改后的表达式
    pub fn try_implicit_cast(
        &self,
        source: Rc<RefCell<Expr>>,
        target_type: Rc<RefCell<Type>>,
    ) -> Result<Rc<RefCell<Expr>>, Diagnostic<usize>> {
        let t = &source.borrow().r#type;
        if is_compatible(Rc::clone(&target_type), Rc::clone(t)) {
            Ok(Rc::clone(&source))
        }
        //尝试所有可能的转换方式
        else if target_type.borrow().is_integer() && t.borrow().is_integer()
            || target_type.borrow().is_integer() && t.borrow().is_real_float()
            || target_type.borrow().is_real_float() && t.borrow().is_real_float()
            || target_type.borrow().is_complex() && t.borrow().is_complex()
            || (target_type.borrow().is_void_ptr() || target_type.borrow().is_pointer())
                && (target_type.borrow().is_void_ptr()
                    || t.borrow().is_pointer()
                    || t.borrow().is_nullptr())
            || target_type.borrow().is_nullptr() && t.borrow().is_nullptr()
            || is_compatible(Rc::clone(&target_type), lvalue_cast(Rc::clone(t)))
            || is_compatible(Rc::clone(&target_type), integer_promote(Rc::clone(t)))
            || is_compatible(Rc::clone(&target_type), func_to_ptr(Rc::clone(t)))
            || is_compatible(Rc::clone(&target_type), array_to_ptr(Rc::clone(t)))
        {
            Ok(Rc::new(RefCell::new(
                self.wrap_implicit_cast(Rc::clone(&source), target_type),
            )))
        } else {
            Err(Diagnostic::error()
                .with_message(format!(
                    "cannot convert '{}' to '{}'",
                    t.borrow().to_string(),
                    target_type.borrow().to_string()
                ))
                .with_label(Label::primary(
                    source.borrow().file_id,
                    source.borrow().span,
                )))
        }
    }

    pub fn visit_expr(&mut self, node: Rc<RefCell<Expr>>) -> Result<(), Diagnostic<usize>> {
        let allow_lvalue_cast = match self.contexts.last() {
            Some(Context::Expr(
                ExprKind::UnaryOp {
                    op:
                        UnaryOpKind::AddressOf
                        | UnaryOpKind::PrefixInc
                        | UnaryOpKind::PostfixInc
                        | UnaryOpKind::PrefixDec
                        | UnaryOpKind::PostfixDec,
                    ..
                }
                | ExprKind::SizeOf { .. },
            )) => false,
            Some(Context::Expr(ExprKind::BinOp {
                op:
                    BinOpKind::Assign
                    | BinOpKind::MulAssign
                    | BinOpKind::DivAssign
                    | BinOpKind::ModAssign
                    | BinOpKind::AddAssign
                    | BinOpKind::SubAssign
                    | BinOpKind::LShiftAssign
                    | BinOpKind::RShiftAssign
                    | BinOpKind::BitAndAssign
                    | BinOpKind::BitOrAssign
                    | BinOpKind::BitXOrAssign,
                left,
                ..
            })) if Rc::ptr_eq(left, &node) => false,
            Some(Context::Expr(ExprKind::MemberAccess {
                is_arrow: false, ..
            })) => false,
            _ => true,
        };
        let allow_array_to_ptr = match self.contexts.last() {
            Some(Context::Expr(
                ExprKind::UnaryOp {
                    op: UnaryOpKind::AddressOf,
                    ..
                }
                | ExprKind::SizeOf { .. },
            )) => false,
            Some(Context::Typeof) => false,
            Some(Context::Init(InitializerKind::Expr(expr))) if Rc::ptr_eq(&node, expr) => false,
            _ => true,
        };
        let allow_func_to_ptr = match self.contexts.last() {
            Some(Context::Expr(
                ExprKind::UnaryOp {
                    op: UnaryOpKind::AddressOf,
                    ..
                }
                | ExprKind::SizeOf { .. },
            )) => false,
            Some(Context::Typeof) => false,
            _ => true,
        };

        self.contexts
            .push(Context::Expr(node.borrow().kind.clone()));

        //先处理完表达式中的类型再对表达式进行类型检查
        match &mut node.borrow_mut().kind {
            ExprKind::SizeOf {
                //只检查没有歧义的情况
                r#type: Some(r#type),
                expr: None,
                decls,
                ..
            }
            | ExprKind::Alignof { r#type, decls } => {
                for decl in decls {
                    self.visit_declaration(Rc::clone(decl))?;
                }
                self.check_type(r#type)?;
            }
            ExprKind::CompoundLiteral { decls, .. }
            | ExprKind::Cast { decls, .. }
            | ExprKind::SizeOf { decls, .. } => {
                for decl in decls {
                    self.visit_declaration(Rc::clone(decl))?;
                }
            }
            ExprKind::GenericSelection { assocs, .. } => {
                for assoc in assocs {
                    for decl in &assoc.borrow().decls {
                        self.visit_declaration(Rc::clone(decl))?;
                    }
                    if let Some(t) = &mut assoc.borrow_mut().r#type {
                        self.check_type(t)?;
                    }
                }
            }
            _ => {}
        }

        let mut errs = Vec::new(); //消歧义时产生的错误
        //消歧义
        let mut new_expr = None;
        let mut new_type = None;
        match &mut *node.borrow_mut() {
            Expr {
                kind:
                    ExprKind::SizeOf {
                        r#type: Some(r#type),
                        expr: Some(expr),
                        ..
                    },
                ..
            } => {
                //消歧义
                match self.visit_expr(Rc::clone(expr)) {
                    Ok(_) => new_expr = Some(Rc::clone(expr)),
                    Err(e) => {
                        errs.push(e);
                    }
                }
                match self.check_type(r#type) {
                    Ok(_) => new_type = Some(Rc::clone(r#type)),
                    Err(e) => {
                        errs.push(e);
                    }
                }
            }
            _ => {}
        }
        match &mut node.borrow_mut().kind {
            ExprKind::SizeOf {
                expr: expr @ Some(_),
                r#type: r#type @ Some(_),
                ..
            } => {
                *expr = new_expr;
                *r#type = new_type;
            }
            _ => {}
        }

        self.check_type(&mut node.borrow_mut().r#type)?;

        {
            let mut node = node.borrow_mut();
            match &node.kind {
                ExprKind::Char { prefix, text } => {
                    let kind;
                    match prefix {
                        EncodePrefix::Default => {
                            let mut value = BigInt::ZERO;
                            for unit in text.as_bytes() {
                                value = value * 256u32 + *unit;
                            }
                            node.value = Variant::Int(value);
                            kind = TypeKind::Int;
                        }
                        EncodePrefix::UTF8 => {
                            if text.as_bytes().len() > 1 {
                                return Err(Diagnostic::error()
                                    .with_message(format!("the character is too large as utf-8"))
                                    .with_label(Label::primary(node.file_id, node.span)));
                            }
                            node.value = Variant::Int(BigInt::from(text.as_bytes()[0]));
                            kind = TypeKind::UnsignedChar;
                        }
                        EncodePrefix::UTF16 => {
                            if text.encode_utf16().count() > 1 {
                                return Err(Diagnostic::error()
                                    .with_message(format!("the character is too large as utf-16"))
                                    .with_label(Label::primary(node.file_id, node.span)));
                            }
                            node.value =
                                Variant::Int(BigInt::from(text.encode_utf16().next().unwrap()));
                            kind = TypeKind::UShort;
                        }
                        EncodePrefix::UTF32 => {
                            if text.chars().count() > 1 {
                                return Err(Diagnostic::error()
                                    .with_message(format!("the character is too large as utf-32"))
                                    .with_label(Label::primary(node.file_id, node.span)));
                            }
                            node.value =
                                Variant::Int(BigInt::from(text.chars().next().unwrap() as u32));
                            kind = TypeKind::UInt;
                        }
                        EncodePrefix::Wide => {
                            let mut value = BigInt::ZERO;
                            for unit in text.chars() {
                                value = value * 256u32 + unit as u32;
                            }
                            node.value = Variant::Int(value);
                            kind = TypeKind::Int;
                        }
                    }
                    node.r#type = Rc::new(RefCell::new(Type {
                        kind: TypeKind::Qualified {
                            qualifiers: vec![TypeQual::Const],
                            r#type: Rc::new(RefCell::new(Type {
                                kind,
                                ..Type::new(node.file_id, node.span)
                            })),
                        },
                        ..Type::new(node.file_id, node.span)
                    }));
                }
                ExprKind::String { prefix, text } => {
                    let element_kind;
                    let mut array = vec![];
                    match prefix {
                        EncodePrefix::Default => {
                            for unit in text.as_bytes() {
                                array.push(Variant::Int(BigInt::from(*unit)));
                            }
                            element_kind = TypeKind::Char;
                        }
                        EncodePrefix::UTF8 => {
                            for unit in text.as_bytes() {
                                array.push(Variant::Int(BigInt::from(*unit)));
                            }
                            element_kind = TypeKind::UnsignedChar;
                        }
                        EncodePrefix::UTF16 => {
                            for unit in text.encode_utf16() {
                                array.push(Variant::Int(BigInt::from(unit)));
                            }
                            element_kind = TypeKind::UShort;
                        }
                        EncodePrefix::UTF32 => {
                            for unit in text.chars() {
                                array.push(Variant::Int(BigInt::from(unit as u32)));
                            }
                            element_kind = TypeKind::UInt;
                        }
                        EncodePrefix::Wide => {
                            for unit in text.chars() {
                                array.push(Variant::Int(BigInt::from(unit as u32)));
                            }
                            element_kind = TypeKind::Int;
                        }
                    }
                    array.push(Variant::Int(BigInt::ZERO)); // '\0'

                    node.r#type = Rc::new(RefCell::new(Type {
                        kind: TypeKind::Array {
                            has_static: false,
                            has_star: false,
                            element_type: Rc::new(RefCell::new(Type {
                                kind: TypeKind::Qualified {
                                    qualifiers: vec![TypeQual::Const],
                                    r#type: Rc::new(RefCell::new(Type {
                                        kind: element_kind,
                                        ..Type::new(node.file_id, node.span)
                                    })),
                                },
                                ..Type::new(node.file_id, node.span)
                            })),
                            len_expr: Some(Rc::new(RefCell::new(Expr::new_const_int(
                                node.file_id,
                                node.span,
                                array.len(),
                                Rc::new(RefCell::new(Type {
                                    kind: TypeKind::ULongLong,
                                    ..Type::new(node.file_id, node.span)
                                })),
                            )))),
                        },
                        ..Type::new(node.file_id, node.span)
                    }));
                    node.value = Variant::Array(array);
                    node.is_lvalue = true;
                }
                ExprKind::Name(name) => {
                    if let Some(t) = &self.cur_symtab.borrow().lookup(Namespace::Ordinary, name) {
                        match &t.borrow().kind {
                            SymbolKind::EnumConst { value } => {
                                node.value = Variant::Int(value.clone())
                            }
                            SymbolKind::Function { .. } => {
                                for attribute in &t.borrow().attributes {
                                    match &attribute.borrow().kind {
                                        AttributeKind::Deprecated { reason } => {
                                            warning(
                                                format!(
                                                    "'{name}' is deprecated{}",
                                                    if let Some(t) = reason {
                                                        format!(": {t}")
                                                    } else {
                                                        "".to_string()
                                                    }
                                                ),
                                                node.file_id,
                                                node.span,
                                                vec![
                                                    Label::secondary(
                                                        attribute.borrow().file_id,
                                                        attribute.borrow().span,
                                                    )
                                                    .with_message("marked here"),
                                                ],
                                            );
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            _ => {}
                        }
                        node.symbol = Some(Rc::clone(t));
                        node.r#type = Rc::clone(&t.borrow().r#type);
                        node.is_lvalue = match &t.borrow().kind {
                            SymbolKind::Object { .. } | SymbolKind::Parameter { .. } => true,
                            SymbolKind::Function { .. } | SymbolKind::EnumConst { .. } => false,
                            _ => {
                                return Err(Diagnostic::error()
                                    .with_message(format!("invalid symbol as expression"))
                                    .with_label(Label::primary(node.file_id, node.span)));
                            }
                        };
                    } else {
                        return Err(Diagnostic::error()
                            .with_message(format!("'{name}' is undefined"))
                            .with_label(Label::primary(node.file_id, node.span)));
                    }
                }
                ExprKind::True => {
                    node.value = Variant::Bool(true);
                    node.r#type = Rc::new(RefCell::new(Type {
                        kind: TypeKind::Bool,
                        ..Type::new(node.file_id, node.span)
                    }));
                }
                ExprKind::False => {
                    node.value = Variant::Bool(false);
                    node.r#type = Rc::new(RefCell::new(Type {
                        kind: TypeKind::Bool,
                        ..Type::new(node.file_id, node.span)
                    }));
                }
                ExprKind::Nullptr => {
                    node.value = Variant::Nullptr;
                    node.r#type = Rc::new(RefCell::new(Type {
                        kind: TypeKind::Nullptr,
                        ..Type::new(node.file_id, node.span)
                    }));
                }
                ExprKind::Integer {
                    base,
                    text,
                    type_suffix,
                } => {
                    let value = BigInt::from_str_radix(
                        if text.len() == 0 { "0" } else { text.as_str() },
                        *base,
                    )
                    .unwrap();

                    let mut candidates;
                    if type_suffix.contains(&"wb".to_string())
                        && type_suffix.contains(&"u".to_string())
                    {
                        candidates = vec![Type {
                            kind: TypeKind::BitInt {
                                unsigned: true,
                                width_expr: Rc::new(RefCell::new(Expr::new_const_int(
                                    node.file_id,
                                    node.span,
                                    value.bits(),
                                    Rc::new(RefCell::new(Type {
                                        kind: TypeKind::ULongLong,
                                        ..Type::new(node.file_id, node.span)
                                    })),
                                ))),
                            },
                            ..Type::new(node.file_id, node.span)
                        }];
                    } else if type_suffix.contains(&"wb".to_string()) {
                        candidates = vec![Type {
                            kind: TypeKind::BitInt {
                                unsigned: false,
                                width_expr: Rc::new(RefCell::new(Expr::new_const_int(
                                    node.file_id,
                                    node.span,
                                    //还有一位符号位
                                    value.bits() + 1,
                                    Rc::new(RefCell::new(Type {
                                        kind: TypeKind::ULongLong,
                                        ..Type::new(node.file_id, node.span)
                                    })),
                                ))),
                            },
                            ..Type::new(node.file_id, node.span)
                        }];
                    } else if type_suffix.contains(&"ll".to_string())
                        && type_suffix.contains(&"u".to_string())
                    {
                        candidates = vec![Type {
                            kind: TypeKind::ULongLong,
                            ..Type::new(node.file_id, node.span)
                        }];
                    } else if type_suffix.contains(&"ll".to_string()) {
                        candidates = vec![Type {
                            kind: TypeKind::LongLong,
                            ..Type::new(node.file_id, node.span)
                        }];
                        if *base != 10 {
                            candidates.push(Type {
                                kind: TypeKind::ULongLong,
                                ..Type::new(node.file_id, node.span)
                            });
                        }
                    } else if type_suffix.contains(&"l".to_string())
                        && type_suffix.contains(&"u".to_string())
                    {
                        candidates = vec![
                            Type {
                                kind: TypeKind::ULong,
                                ..Type::new(node.file_id, node.span)
                            },
                            Type {
                                kind: TypeKind::ULongLong,
                                ..Type::new(node.file_id, node.span)
                            },
                        ];
                    } else if type_suffix.contains(&"l".to_string()) {
                        candidates = vec![
                            Type {
                                kind: TypeKind::Long,
                                ..Type::new(node.file_id, node.span)
                            },
                            Type {
                                kind: TypeKind::ULong,
                                ..Type::new(node.file_id, node.span)
                            },
                            Type {
                                kind: TypeKind::LongLong,
                                ..Type::new(node.file_id, node.span)
                            },
                        ];
                        if *base != 10 {
                            candidates.push(Type {
                                kind: TypeKind::ULongLong,
                                ..Type::new(node.file_id, node.span)
                            });
                        }
                    } else if type_suffix.contains(&"u".to_string()) {
                        candidates = vec![
                            Type {
                                kind: TypeKind::UInt,
                                ..Type::new(node.file_id, node.span)
                            },
                            Type {
                                kind: TypeKind::ULong,
                                ..Type::new(node.file_id, node.span)
                            },
                            Type {
                                kind: TypeKind::ULongLong,
                                ..Type::new(node.file_id, node.span)
                            },
                        ];
                    } else {
                        candidates = vec![
                            Type {
                                kind: TypeKind::Int,
                                ..Type::new(node.file_id, node.span)
                            },
                            Type {
                                kind: TypeKind::Long,
                                ..Type::new(node.file_id, node.span)
                            },
                            Type {
                                kind: TypeKind::ULong,
                                ..Type::new(node.file_id, node.span)
                            },
                            Type {
                                kind: TypeKind::LongLong,
                                ..Type::new(node.file_id, node.span)
                            },
                        ];
                        if *base != 10 {
                            candidates.extend(vec![
                                Type {
                                    kind: TypeKind::UInt,
                                    ..Type::new(node.file_id, node.span)
                                },
                                Type {
                                    kind: TypeKind::ULongLong,
                                    ..Type::new(node.file_id, node.span)
                                },
                            ]);
                        }
                    }

                    let mut r#type = None;

                    for candidate in candidates {
                        let Some((min, max)) = candidate.range() else {
                            continue;
                        };
                        if min <= value && value <= max {
                            r#type = Some(candidate);
                            break;
                        }
                    }

                    if let Some(t) = r#type {
                        node.r#type = Rc::new(RefCell::new(t));
                    } else {
                        return Err(Diagnostic::error()
                            .with_message(format!("the integer is too large"))
                            .with_label(Label::primary(node.file_id, node.span)));
                    }

                    node.value = Variant::Int(value.clone());
                }
                ExprKind::Float {
                    base,
                    digits,
                    exp_base,
                    exponent,
                    type_suffix,
                } => {
                    let mut base = BigRational::from_u32(*base).unwrap();
                    let mut weight = BigRational::from_u64(1).unwrap();
                    let dot_index = digits.find(".").unwrap_or(digits.len());
                    let mut value = BigRational::from_u64(0).unwrap();
                    let digits = digits
                        .chars()
                        .map(|x| match x {
                            '0'..='9' => x as u64 - '0' as u64,
                            'a'..='z' => 10 + x as u64 - 'a' as u64,
                            'A'..='Z' => 10 + x as u64 - 'Z' as u64,
                            _ => x as u64,
                        })
                        .collect::<Vec<u64>>();

                    //处理有效数字
                    for i in (0..dot_index).rev() {
                        if let Some(x) = digits.get(i) {
                            if *x == '-' as u64 {
                                value = -value;
                            } else if *x == '+' as u64 {
                            } else {
                                value = value + &weight * BigRational::from_u64(*x).unwrap();
                                weight = weight * &base;
                            }
                        }
                    }
                    base = BigRational::from_str(format!("1/{base}").as_str()).unwrap();
                    weight = base.clone();
                    for i in dot_index + 1..digits.len() {
                        if let Some(x) = digits.get(i) {
                            value = value + &weight * BigRational::from_u64(*x).unwrap();
                            weight = weight * &base;
                        }
                    }
                    //处理指数
                    let mut exponent = exponent.chars().collect::<Vec<char>>();
                    if exponent[0] == '-' {
                        base = BigRational::from_str(format!("1/{exp_base}").as_str()).unwrap();
                        exponent = exponent[1..].to_vec();
                    } else {
                        base = BigRational::from_u32(*exp_base).unwrap();
                        if exponent[0] == '+' {
                            exponent = exponent[1..].to_vec();
                        }
                    }
                    let exponent = BigUint::from_str(&exponent.iter().collect::<String>()).unwrap();
                    for i in 0..exponent.bits() {
                        if exponent.bit(i) {
                            value = value * &base;
                        }
                        base = &base * &base;
                    }

                    node.r#type = Rc::new(RefCell::new(Type {
                        kind: if type_suffix.contains(&"df".to_string()) {
                            TypeKind::Decimal32
                        } else if type_suffix.contains(&"dd".to_string()) {
                            TypeKind::Decimal64
                        } else if type_suffix.contains(&"dl".to_string()) {
                            TypeKind::Decimal128
                        } else if type_suffix.contains(&"l".to_string()) {
                            TypeKind::LongDouble
                        } else if type_suffix.contains(&"f".to_string()) {
                            TypeKind::Float
                        } else {
                            TypeKind::Double
                        },
                        ..Type::new(node.file_id, node.span)
                    }));
                    node.value = Variant::Rational(value);
                }
                ExprKind::Cast { target, .. } => {
                    self.visit_expr(Rc::clone(target))?;

                    if !(match node.r#type.borrow().kind {
                        TypeKind::Void => true,
                        _ => false,
                    } || is_compatible(
                        Rc::clone(&node.r#type),
                        Rc::clone(&target.borrow().r#type),
                    )) {
                        let a = node.r#type.borrow();
                        let target_ref = target.borrow();
                        let b = target_ref.r#type.borrow();
                        if !(a.is_integer() && b.is_pointer()
                            || a.is_pointer() && b.is_integer()
                            || a.is_pointer() && b.is_pointer())
                        {
                            self.try_implicit_cast(Rc::clone(target), Rc::clone(&node.r#type))?;
                        }
                    }

                    let value = target.borrow().value.clone();
                    let symbol = target.borrow().symbol.clone();
                    node.value = value;
                    node.symbol = symbol;
                }
                ExprKind::GenericSelection {
                    control_expr,
                    assocs,
                } => {
                    self.visit_expr(Rc::clone(control_expr))?;
                    let control_type = func_to_ptr(array_to_ptr(lvalue_cast(Rc::clone(
                        &control_expr.borrow().r#type,
                    ))));

                    let mut select_index = None;
                    for (i, assoc) in assocs.iter().enumerate() {
                        if let Some(r#type) = &assoc.borrow().r#type {
                            if is_compatible(Rc::clone(r#type), Rc::clone(&control_type)) {
                                select_index = Some(i);
                                break;
                            }
                        } else {
                            //default
                            select_index = Some(i);
                            break;
                        }
                    }
                    if let Some(i) = select_index {
                        let expr = Rc::clone(&assocs.get(i).unwrap().borrow().expr);
                        self.visit_expr(Rc::clone(&expr))?;
                        node.r#type = expr.borrow().r#type.clone();
                        node.value = expr.borrow().value.clone();
                        node.symbol = expr.borrow().symbol.clone();

                        match &mut node.kind {
                            ExprKind::GenericSelection { assocs, .. } => {
                                assocs.get_mut(i).unwrap().borrow_mut().is_selected = true;
                            }
                            _ => unreachable!(),
                        }
                    } else {
                        return Err(Diagnostic::error()
                            .with_message(format!("cannot find suitable association"))
                            .with_label(Label::primary(node.file_id, node.span)));
                    }
                }
                ExprKind::FunctionCall { target, arguments } => {
                    self.visit_expr(Rc::clone(target))?;
                    for argument in arguments {
                        self.visit_expr(Rc::clone(argument))?;
                    }

                    let mut func_type = None;
                    match &target.borrow().r#type.borrow().kind {
                        TypeKind::Function { .. } => {
                            func_type = Some(Rc::clone(&target.borrow().r#type))
                        }
                        TypeKind::Pointer(pointee) => match &pointee.borrow().kind {
                            TypeKind::Function { .. } => func_type = Some(Rc::clone(pointee)),
                            _ => {}
                        },
                        _ => {}
                    }
                    if let Some(func_type) = func_type {
                        let TypeKind::Function {
                            return_type,
                            parameters_type,
                            has_varparam,
                        } = &func_type.borrow().kind
                        else {
                            unreachable!();
                        };

                        if parameters_type.len() > arguments.len() {
                            return Err(Diagnostic::error()
                                .with_message(format!("too few arguments to function call"))
                                .with_label(Label::primary(node.file_id, node.span)));
                        }
                        if !*has_varparam && parameters_type.len() < arguments.len() {
                            return Err(Diagnostic::error()
                                .with_message(format!("too many arguments to function call"))
                                .with_label(Label::primary(node.file_id, node.span)));
                        }
                        node.r#type = Rc::clone(return_type);

                        let ExprKind::FunctionCall { arguments, .. } = &mut node.kind else {
                            unreachable!();
                        };
                        let mut i = 0;
                        while i < parameters_type.len() {
                            let argument = arguments.get_mut(i).unwrap();
                            let parameter_type = parameters_type.get(i).unwrap();

                            let origin;
                            match &argument.borrow().kind {
                                ExprKind::Cast {
                                    is_implicit: true,
                                    target,
                                    ..
                                } => origin = Rc::clone(target),
                                _ => origin = Rc::clone(argument),
                            }

                            match &origin.borrow().r#type.borrow().kind {
                                TypeKind::Array {
                                    len_expr: Some(len_expr),
                                    ..
                                } => {
                                    let mut attributes = parameter_type.borrow().attributes.clone();
                                    match &parameter_type.borrow().kind {
                                        //针对 T[const static C] 之类的情况
                                        TypeKind::Qualified { r#type, .. } => {
                                            attributes.extend(r#type.borrow().attributes.clone())
                                        }
                                        _ => {}
                                    }
                                    for attribute in &attributes {
                                        let AttributeKind::PtrFromArray { array_type } =
                                            &attribute.borrow().kind
                                        else {
                                            continue;
                                        };
                                        let TypeKind::Array {
                                            has_static: true,
                                            len_expr: Some(min_len_expr),
                                            ..
                                        } = &array_type.borrow().kind
                                        else {
                                            continue;
                                        };
                                        if len_expr.borrow().value < min_len_expr.borrow().value {
                                            warning(
                                                format!(
                                                    "array argument is too small; contains {} elements, callee requires at least {}",
                                                    len_expr.borrow().value,
                                                    min_len_expr.borrow().value
                                                ),
                                                argument.borrow().file_id,
                                                argument.borrow().span,
                                                vec![],
                                            );
                                        }
                                    }
                                }
                                _ => {}
                            }

                            *argument = self.try_implicit_cast(
                                Rc::clone(argument),
                                remove_qualifier(Rc::clone(parameter_type)),
                            )?;
                            i = i + 1;
                        }
                        //属于省略号的参数
                        while i < arguments.len() {
                            let argument = arguments.get_mut(i).unwrap();
                            let r#type = Rc::clone(&argument.borrow().r#type);
                            if r#type.borrow().is_integer() {
                                let target_type =
                                    integer_promote(Rc::clone(&argument.borrow().r#type));
                                *argument =
                                    self.try_implicit_cast(Rc::clone(argument), target_type)?;
                            } else if let TypeKind::Float = r#type.borrow().kind {
                                let target_type = Rc::new(RefCell::new(Type {
                                    kind: TypeKind::Double,
                                    ..Type::new(argument.borrow().file_id, argument.borrow().span)
                                }));
                                *argument =
                                    self.try_implicit_cast(Rc::clone(argument), target_type)?;
                            }
                            i = i + 1;
                        }
                    } else {
                        return Err(Diagnostic::error()
                            .with_message(format!(
                                "{} is not a function or a function pointer",
                                target.borrow().r#type.borrow().to_string()
                            ))
                            .with_label(Label::primary(
                                target.borrow().file_id,
                                target.borrow().span,
                            )));
                    }
                }
                ExprKind::Subscript { target, index } => {
                    self.visit_expr(Rc::clone(target))?;
                    self.visit_expr(Rc::clone(index))?;

                    let mut value = Variant::Unknown;
                    let mut i = Variant::Unknown;
                    let mut r#type = None;
                    match (
                        &target.borrow().r#type.borrow().kind,
                        &index.borrow().r#type.borrow().kind,
                    ) {
                        (TypeKind::Pointer(pointee), x) if x.is_integer() => {
                            value = target.borrow().value.clone();
                            i = index.borrow().value.clone();
                            r#type = Some(Rc::clone(pointee));
                        }
                        (x, TypeKind::Pointer(pointee)) if x.is_integer() => {
                            value = index.borrow().value.clone();
                            i = target.borrow().value.clone();
                            r#type = Some(Rc::clone(pointee));
                        }
                        (TypeKind::Array { element_type, .. }, x) if x.is_integer() => {
                            value = target.borrow().value.clone();
                            i = index.borrow().value.clone();
                            r#type = Some(Rc::clone(element_type));
                        }
                        (x, TypeKind::Array { element_type, .. }) if x.is_integer() => {
                            value = index.borrow().value.clone();
                            i = target.borrow().value.clone();
                            r#type = Some(Rc::clone(element_type));
                        }
                        _ => {}
                    }
                    if let Some(t) = r#type {
                        node.r#type = t;
                        node.is_lvalue = true;
                        node.value = value.get(&i).clone();
                    } else {
                        return Err(Diagnostic::error().with_message(format!(
                                "subscripted value must be a pointer or array and index must be an integer"
                            )).with_label(Label::primary(node.file_id,node.span)));
                    }
                }
                ExprKind::MemberAccess {
                    target,
                    is_arrow,
                    name,
                } => {
                    self.visit_expr(Rc::clone(target))?;

                    let is_lvalue;
                    let target_type;
                    if *is_arrow {
                        is_lvalue = true;
                        match &target.borrow().r#type.borrow().kind {
                            TypeKind::Pointer(pointee) => target_type = Rc::clone(pointee),
                            _ => {
                                return Err(Diagnostic::error()
                                    .with_message(format!(
                                        "'{}' is not a pointer",
                                        target.borrow().r#type.borrow().to_string()
                                    ))
                                    .with_label(Label::primary(
                                        target.borrow().file_id,
                                        target.borrow().span,
                                    )));
                            }
                        }
                    } else {
                        is_lvalue = target.borrow().is_lvalue;
                        target_type = Rc::clone(&target.borrow().r#type);
                    }

                    let mut qualifiers = vec![];
                    let mut record_type = None;
                    match &target_type.borrow().kind {
                        TypeKind::Qualified {
                            qualifiers: quals,
                            r#type,
                        } => match &r#type.borrow().kind {
                            TypeKind::Record { .. } => {
                                record_type = Some(Rc::clone(r#type));
                                qualifiers.extend(quals.clone());
                            }
                            _ => {}
                        },
                        TypeKind::Record { .. } => {
                            record_type = Some(Rc::clone(&target.borrow().r#type));
                        }
                        _ => {}
                    }
                    if let Some(t) = record_type {
                        let TypeKind::Record { members, .. } = &t.borrow().kind else {
                            unreachable!();
                        };
                        if let Some(members) = members {
                            if let Some(member) = members.get(name) {
                                node.symbol = Some(Rc::clone(member));
                                if qualifiers.len() > 0 {
                                    node.r#type = Rc::new(RefCell::new(Type {
                                        kind: TypeKind::Qualified {
                                            qualifiers,
                                            r#type: Rc::clone(&member.borrow().r#type),
                                        },
                                        ..Type::new(node.file_id, node.span)
                                    }));
                                } else {
                                    node.r#type = Rc::clone(&member.borrow().r#type);
                                }
                            } else {
                                return Err(Diagnostic::error()
                                    .with_message(format!(
                                        "no member named '{name}' in '{}'",
                                        t.borrow().to_string()
                                    ))
                                    .with_label(Label::primary(
                                        target.borrow().file_id,
                                        target.borrow().span,
                                    )));
                            }
                        } else {
                            return Err(Diagnostic::error()
                                .with_message(format!("'{}' is incomplete", t.borrow().to_string()))
                                .with_label(Label::primary(
                                    target.borrow().file_id,
                                    target.borrow().span,
                                )));
                        }
                    } else {
                        return Err(Diagnostic::error()
                            .with_message(format!(
                                "'{}' is not a struct or union",
                                target.borrow().r#type.borrow().to_string()
                            ))
                            .with_label(Label::primary(
                                target.borrow().file_id,
                                target.borrow().span,
                            )));
                    }
                    node.is_lvalue = is_lvalue;
                }
                ExprKind::UnaryOp { op, operand } => {
                    self.visit_expr(Rc::clone(operand))?;
                    match op {
                        UnaryOpKind::Not => {
                            if operand.borrow().r#type.borrow().is_scale() {
                                let value = !operand.borrow().value.clone();
                                node.value = value;
                                node.r#type = Rc::new(RefCell::new(Type {
                                    kind: TypeKind::Int,
                                    ..Type::new(node.file_id, node.span)
                                }))
                            } else {
                                return Err(Diagnostic::error()
                                    .with_message(format!(
                                        "'{}' is not a scale type",
                                        operand.borrow().r#type.borrow().to_string()
                                    ))
                                    .with_label(Label::primary(
                                        operand.borrow().file_id,
                                        operand.borrow().span,
                                    )));
                            }
                        }
                        UnaryOpKind::Positive | UnaryOpKind::Negative => {
                            if operand.borrow().r#type.borrow().is_arithmetic() {
                                let promote_type =
                                    integer_promote(Rc::clone(&operand.borrow().r#type));
                                let mut value = operand.borrow().value.clone();
                                if let UnaryOpKind::Negative = op {
                                    value = -value;
                                }

                                match &mut node.kind {
                                    ExprKind::UnaryOp { operand, .. } => {
                                        *operand = self.try_implicit_cast(
                                            Rc::clone(operand),
                                            Rc::clone(&promote_type),
                                        )?;
                                    }
                                    _ => {}
                                }

                                node.value = value;
                                node.r#type = promote_type;
                            } else {
                                return Err(Diagnostic::error()
                                    .with_message(format!(
                                        "'{}' is not a arithmetic type",
                                        operand.borrow().r#type.borrow().to_string()
                                    ))
                                    .with_label(Label::primary(
                                        operand.borrow().file_id,
                                        operand.borrow().span,
                                    )));
                            }
                        }
                        UnaryOpKind::BitNot => {
                            if operand.borrow().r#type.borrow().is_integer() {
                                let promote_type =
                                    integer_promote(Rc::clone(&operand.borrow().r#type));
                                let value = !operand.borrow().value.clone();

                                match &mut node.kind {
                                    ExprKind::UnaryOp { operand, .. } => {
                                        *operand = self.try_implicit_cast(
                                            Rc::clone(operand),
                                            Rc::clone(&promote_type),
                                        )?;
                                    }
                                    _ => {}
                                }

                                node.value = value;
                                node.r#type = promote_type;
                            } else {
                                return Err(Diagnostic::error()
                                    .with_message(format!(
                                        "'{}' is not a integer type",
                                        operand.borrow().r#type.borrow().to_string()
                                    ))
                                    .with_label(Label::primary(
                                        operand.borrow().file_id,
                                        operand.borrow().span,
                                    )));
                            }
                        }
                        UnaryOpKind::PrefixInc
                        | UnaryOpKind::PostfixInc
                        | UnaryOpKind::PrefixDec
                        | UnaryOpKind::PostfixDec => {
                            let r#type = Rc::clone(&operand.borrow().r#type);
                            if !operand.borrow().is_lvalue || !r#type.borrow().can_modify() {
                                return Err(Diagnostic::error()
                                    .with_message(format!("invalid operand"))
                                    .with_label(Label::primary(
                                        operand.borrow().file_id,
                                        operand.borrow().span,
                                    )));
                            } else if !(r#type.borrow().is_integer()
                                || r#type.borrow().is_real_float()
                                || r#type.borrow().is_pointer())
                            {
                                return Err(Diagnostic::error()
                                    .with_message(format!(
                                        "'{}' is not a integer, real floating or pointer",
                                        r#type.borrow().to_string()
                                    ))
                                    .with_label(Label::primary(
                                        operand.borrow().file_id,
                                        operand.borrow().span,
                                    )));
                            } else {
                                let value = if let UnaryOpKind::PrefixInc
                                | UnaryOpKind::PostfixInc = op
                                {
                                    operand.borrow().value.clone() + Variant::Int(BigInt::from(1))
                                } else {
                                    operand.borrow().value.clone() - Variant::Int(BigInt::from(1))
                                };

                                node.value = value;
                                node.r#type = r#type;
                            }
                        }
                        UnaryOpKind::AddressOf => {
                            if match &operand.borrow().kind {
                                //解引用不一定是左值
                                ExprKind::UnaryOp {
                                    op: UnaryOpKind::Dereference,
                                    ..
                                } => true,
                                _ => match operand.borrow().r#type.borrow().kind {
                                    TypeKind::Function { .. } => true,
                                    _ => operand.borrow().is_lvalue,
                                },
                            } {
                                match operand.borrow().symbol {
                                    Some(ref symbol) => match &symbol.borrow().kind {
                                        SymbolKind::Member { bit_field: Some(_) } => {
                                            return Err(Diagnostic::error()
                                                .with_message(format!(
                                                    "cannot take address of bit-field"
                                                ))
                                                .with_label(Label::primary(
                                                    node.file_id,
                                                    node.span,
                                                )));
                                        }
                                        SymbolKind::Object { storage_classes }
                                        | SymbolKind::Parameter { storage_classes } => {
                                            for storage_class in storage_classes {
                                                match &storage_class.kind {
                                                    StorageClassKind::Register => {
                                                        return Err(Diagnostic::error().with_message(format!(
                                                                "cannot take address of register variable"
                                                            )).with_label(Label::primary(node.file_id,node.span)));
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        }
                                        _ => {}
                                    },
                                    None => {}
                                }
                                let r#type = Rc::new(RefCell::new(Type {
                                    kind: TypeKind::Pointer(Rc::clone(&operand.borrow().r#type)),
                                    ..Type::new(node.file_id, node.span)
                                }));
                                node.r#type = r#type;
                            } else {
                                return Err(Diagnostic::error().with_message(format!(
                                        "the operand of '&' must be a function, lvalue or dereference"
                                    )).with_label(Label::primary(operand.borrow().file_id,operand.borrow().span)));
                            }
                        }
                        UnaryOpKind::Dereference => {
                            if operand.borrow().r#type.borrow().is_pointer() {
                                let is_lvalue = match &operand.borrow().kind {
                                    //如果原先的operand是lvalue, 那么它会进行左值转换从而匹配这个模式
                                    ExprKind::Cast {
                                        is_implicit: true,
                                        target,
                                        ..
                                    } => target.borrow().is_lvalue,
                                    _ => operand.borrow().is_lvalue,
                                };
                                let r#type = pointee(Rc::clone(&operand.borrow().r#type)).unwrap();

                                node.r#type = r#type;
                                node.is_lvalue = is_lvalue;
                            } else {
                                return Err(Diagnostic::error()
                                    .with_message(format!(
                                        "'{}' is not a pointer",
                                        operand.borrow().r#type.borrow().to_string()
                                    ))
                                    .with_label(Label::primary(
                                        operand.borrow().file_id,
                                        operand.borrow().span,
                                    )));
                            }
                        }
                    }
                }
                ExprKind::BinOp { op, left, right } => {
                    self.visit_expr(Rc::clone(left))?;
                    self.visit_expr(Rc::clone(right))?;

                    match op {
                        BinOpKind::And | BinOpKind::Or => {
                            if !left.borrow().r#type.borrow().is_scale() {
                                return Err(Diagnostic::error()
                                    .with_message(format!(
                                        "the operator of '{op}' must have a scale type"
                                    ))
                                    .with_label(Label::primary(
                                        left.borrow().file_id,
                                        left.borrow().span,
                                    )));
                            } else if !right.borrow().r#type.borrow().is_scale() {
                                return Err(Diagnostic::error()
                                    .with_message(format!(
                                        "the operator of '{op}' must have a scale type"
                                    ))
                                    .with_label(Label::primary(
                                        right.borrow().file_id,
                                        right.borrow().span,
                                    )));
                            } else {
                                let (x, y) =
                                    (left.borrow().value.clone(), right.borrow().value.clone());
                                let value = match op {
                                    BinOpKind::And => x.and(&y),
                                    BinOpKind::Or => x.or(&y),
                                    _ => unreachable!(),
                                };

                                node.value = value;
                                node.r#type = Rc::new(RefCell::new(Type {
                                    kind: TypeKind::Int,
                                    ..Type::new(node.file_id, node.span)
                                }));
                            }
                        }
                        BinOpKind::Lt | BinOpKind::Le | BinOpKind::Gt | BinOpKind::Ge => {
                            if left.borrow().r#type.borrow().is_real()
                                && right.borrow().r#type.borrow().is_real()
                            {
                                let (a, b) = usual_arith_cast(
                                    Rc::clone(&left.borrow().r#type),
                                    Rc::clone(&right.borrow().r#type),
                                )?;

                                let (x, y) =
                                    (left.borrow().value.clone(), right.borrow().value.clone());
                                let value = match op {
                                    BinOpKind::Lt => x.lt(&y),
                                    BinOpKind::Le => x.le(&y),
                                    BinOpKind::Gt => x.gt(&y),
                                    BinOpKind::Ge => x.ge(&y),
                                    _ => unreachable!(),
                                };
                                node.value = value;

                                match &mut node.kind {
                                    ExprKind::BinOp { left, right, .. } => {
                                        *left = self.try_implicit_cast(Rc::clone(left), a)?;
                                        *right = self.try_implicit_cast(Rc::clone(right), b)?;
                                    }
                                    _ => unreachable!(),
                                }
                            } else if left.borrow().r#type.borrow().is_pointer()
                                && right.borrow().r#type.borrow().is_pointer()
                            {
                                if !is_compatible(
                                    pointee(Rc::clone(&left.borrow().r#type)).unwrap(),
                                    pointee(Rc::clone(&right.borrow().r#type)).unwrap(),
                                ) {
                                    return Err(Diagnostic::error()
                                        .with_message(format!(
                                            "'{}' is not compatible with '{}'",
                                            left.borrow().r#type.borrow().to_string(),
                                            right.borrow().r#type.borrow().to_string()
                                        ))
                                        .with_label(Label::primary(
                                            left.borrow().file_id,
                                            left.borrow().span,
                                        )));
                                }
                            } else {
                                return Err(Diagnostic::error().with_message(format!(
                                        "the operator of '{op}' must have an real type or is a pointer"
                                    )).with_label(Label::primary(left.borrow().file_id,left.borrow().span)).with_message("the left operand")
                                .with_label(Label::primary(right.borrow().file_id,right.borrow().span).with_message("the right operand")));
                            }
                            node.r#type = Rc::new(RefCell::new(Type {
                                kind: TypeKind::Int,
                                ..Type::new(node.file_id, node.span)
                            }));
                        }
                        BinOpKind::Eq | BinOpKind::Neq => {
                            if left.borrow().r#type.borrow().is_arithmetic()
                                && right.borrow().r#type.borrow().is_arithmetic()
                            {
                                let (a, b) = usual_arith_cast(
                                    Rc::clone(&left.borrow().r#type),
                                    Rc::clone(&right.borrow().r#type),
                                )?;

                                let (x, y) =
                                    (left.borrow().value.clone(), right.borrow().value.clone());
                                let value = match op {
                                    BinOpKind::Eq => x.eq(&y),
                                    BinOpKind::Neq => x.neq(&y),
                                    _ => unreachable!(),
                                };
                                node.value = value;

                                match &mut node.kind {
                                    ExprKind::BinOp { left, right, .. } => {
                                        *left = self.try_implicit_cast(Rc::clone(left), a)?;
                                        *right = self.try_implicit_cast(Rc::clone(right), b)?;
                                    }
                                    _ => unreachable!(),
                                }
                            } else if left.borrow().r#type.borrow().is_pointer()
                                || left.borrow().r#type.borrow().is_nullptr()
                                    && right.borrow().r#type.borrow().is_pointer()
                                || right.borrow().r#type.borrow().is_nullptr()
                            {
                                match &mut node.kind {
                                    ExprKind::BinOp { left, right, .. } => {
                                        let new_left = self.try_implicit_cast(
                                            Rc::clone(left),
                                            Rc::new(RefCell::new(Type {
                                                kind: TypeKind::Pointer(Rc::new(RefCell::new(
                                                    Type {
                                                        kind: TypeKind::Void,
                                                        ..Type::new(
                                                            left.borrow().file_id,
                                                            left.borrow().span,
                                                        )
                                                    },
                                                ))),
                                                ..Type::new(
                                                    left.borrow().file_id,
                                                    left.borrow().span,
                                                )
                                            })),
                                        )?;
                                        let new_right = self.try_implicit_cast(
                                            Rc::clone(right),
                                            Rc::new(RefCell::new(Type {
                                                kind: TypeKind::Pointer(Rc::new(RefCell::new(
                                                    Type {
                                                        kind: TypeKind::Void,
                                                        ..Type::new(
                                                            right.borrow().file_id,
                                                            right.borrow().span,
                                                        )
                                                    },
                                                ))),
                                                ..Type::new(
                                                    right.borrow().file_id,
                                                    right.borrow().span,
                                                )
                                            })),
                                        )?;
                                        *left = new_left;
                                        *right = new_right;
                                    }
                                    _ => unreachable!(),
                                }
                                node.r#type = Rc::new(RefCell::new(Type {
                                    kind: TypeKind::Pointer(Rc::new(RefCell::new(Type {
                                        kind: TypeKind::Void,
                                        ..Type::new(node.file_id, node.span)
                                    }))),
                                    ..Type::new(node.file_id, node.span)
                                }));
                            } else {
                                return Err(Diagnostic::error().with_message(format!(
                                        "the operator of '{op}' must have an arithmetic type or is a pointer"
                                    )).with_label(Label::primary(left.borrow().file_id,left.borrow().span)).with_message("the left operand")
                                .with_label(Label::primary(right.borrow().file_id,right.borrow().span).with_message("the right operand")));
                            }
                            node.r#type = Rc::new(RefCell::new(Type {
                                kind: TypeKind::Int,
                                ..Type::new(node.file_id, node.span)
                            }));
                        }
                        BinOpKind::Add | BinOpKind::Sub => {
                            if left.borrow().r#type.borrow().is_arithmetic()
                                && right.borrow().r#type.borrow().is_arithmetic()
                            {
                                let (a, b) = usual_arith_cast(
                                    Rc::clone(&left.borrow().r#type),
                                    Rc::clone(&right.borrow().r#type),
                                )?;

                                let (x, y) =
                                    (left.borrow().value.clone(), right.borrow().value.clone());
                                let value = match op {
                                    BinOpKind::Add => x + y,
                                    BinOpKind::Sub => x - y,
                                    _ => unreachable!(),
                                };
                                node.value = value;

                                match &mut node.kind {
                                    ExprKind::BinOp { left, right, .. } => {
                                        *left =
                                            self.try_implicit_cast(Rc::clone(left), Rc::clone(&a))?;
                                        *right = self
                                            .try_implicit_cast(Rc::clone(right), Rc::clone(&b))?;
                                    }
                                    _ => unreachable!(),
                                }
                                node.r#type = arith_result_type(a, b);
                            }
                            //integer肯定是算术类型, 所以执行到这里时两个操作数要么一个是指针一个是整数, 要么都是指针
                            else if left.borrow().r#type.borrow().is_pointer()
                                || left.borrow().r#type.borrow().is_integer()
                                    && right.borrow().r#type.borrow().is_pointer()
                                || right.borrow().r#type.borrow().is_integer()
                            {
                                if let BinOpKind::Sub = op
                                    && left.borrow().r#type.borrow().is_integer()
                                {
                                    return Err(Diagnostic::error().with_message( format!(
                                            "the left operand of {op} cannot be an when the right operand is a pointer"
                                        )).with_label(Label::primary(left.borrow().file_id,left.borrow().span)));
                                }

                                let r#type = if left.borrow().r#type.borrow().is_pointer() {
                                    Rc::clone(&left.borrow().r#type)
                                } else {
                                    Rc::clone(&right.borrow().r#type)
                                };
                                node.r#type = r#type;
                            } else {
                                return Err(Diagnostic::error().with_message( format!(
                                        "the operator of '{op}' must have an arithmetic type or is a pointer"
                                    )).with_label(Label::primary(left.borrow().file_id,left.borrow().span)).with_message("the left operand")
                                .with_label(Label::primary(right.borrow().file_id,right.borrow().span).with_message("the right operand")));
                            }
                        }
                        BinOpKind::Mul | BinOpKind::Div | BinOpKind::Mod => {
                            //对于取余运算只能是整数, 但这里并没有判断
                            if left.borrow().r#type.borrow().is_arithmetic()
                                && right.borrow().r#type.borrow().is_arithmetic()
                            {
                                let (a, b) = usual_arith_cast(
                                    Rc::clone(&left.borrow().r#type),
                                    Rc::clone(&right.borrow().r#type),
                                )?;

                                let (x, y) =
                                    (left.borrow().value.clone(), right.borrow().value.clone());
                                let value = match op {
                                    BinOpKind::Mul => x * y,
                                    BinOpKind::Div => x / y,
                                    BinOpKind::Mod => x % y,
                                    _ => unreachable!(),
                                };
                                node.value = value;

                                match &mut node.kind {
                                    ExprKind::BinOp { left, right, .. } => {
                                        *left =
                                            self.try_implicit_cast(Rc::clone(left), Rc::clone(&a))?;
                                        *right = self
                                            .try_implicit_cast(Rc::clone(right), Rc::clone(&b))?;
                                    }
                                    _ => unreachable!(),
                                }
                                node.r#type = arith_result_type(a, b);
                            } else {
                                return Err(Diagnostic::error()
                                    .with_message(format!(
                                        "the operator of '{op}' must have an arithmetic type"
                                    ))
                                    .with_label(Label::primary(
                                        left.borrow().file_id,
                                        left.borrow().span,
                                    ))
                                    .with_message("the left operand")
                                    .with_label(
                                        Label::primary(right.borrow().file_id, right.borrow().span)
                                            .with_message("the right operand"),
                                    ));
                            }
                        }
                        BinOpKind::BitAnd | BinOpKind::BitOr | BinOpKind::BitXOr => {
                            if left.borrow().r#type.borrow().is_integer()
                                && right.borrow().r#type.borrow().is_integer()
                            {
                                let (a, b) = usual_arith_cast(
                                    Rc::clone(&left.borrow().r#type),
                                    Rc::clone(&right.borrow().r#type),
                                )?;

                                let (x, y) =
                                    (left.borrow().value.clone(), right.borrow().value.clone());
                                let value = match op {
                                    BinOpKind::BitAnd => x & y,
                                    BinOpKind::BitOr => x | y,
                                    BinOpKind::BitXOr => x ^ y,
                                    _ => unreachable!(),
                                };
                                node.value = value;

                                match &mut node.kind {
                                    ExprKind::BinOp { left, right, .. } => {
                                        *left =
                                            self.try_implicit_cast(Rc::clone(left), Rc::clone(&a))?;
                                        *right = self
                                            .try_implicit_cast(Rc::clone(right), Rc::clone(&b))?;
                                    }
                                    _ => unreachable!(),
                                }
                                node.r#type = arith_result_type(a, b);
                            } else {
                                return Err(Diagnostic::error()
                                    .with_message(format!(
                                        "the operator of '{op}' must have an integer type"
                                    ))
                                    .with_label(Label::primary(
                                        left.borrow().file_id,
                                        left.borrow().span,
                                    ))
                                    .with_message("the left operand")
                                    .with_label(
                                        Label::primary(right.borrow().file_id, right.borrow().span)
                                            .with_message("the right operand"),
                                    ));
                            }
                        }
                        BinOpKind::LShift | BinOpKind::RShift => {
                            if left.borrow().r#type.borrow().is_integer()
                                && right.borrow().r#type.borrow().is_integer()
                            {
                                let a = integer_promote(Rc::clone(&left.borrow().r#type));
                                let b = integer_promote(Rc::clone(&right.borrow().r#type));

                                let (x, y) =
                                    (left.borrow().value.clone(), right.borrow().value.clone());
                                let value = match op {
                                    BinOpKind::LShift => x << y,
                                    BinOpKind::RShift => x >> y,
                                    _ => unreachable!(),
                                };
                                node.value = value;

                                match &mut node.kind {
                                    ExprKind::BinOp { left, right, .. } => {
                                        *left =
                                            self.try_implicit_cast(Rc::clone(left), Rc::clone(&a))?;
                                        *right = self
                                            .try_implicit_cast(Rc::clone(right), Rc::clone(&b))?;
                                    }
                                    _ => unreachable!(),
                                }
                                node.r#type = a;
                            } else {
                                return Err(Diagnostic::error()
                                    .with_message(format!(
                                        "the operator of '{op}' must have an integer type"
                                    ))
                                    .with_label(Label::primary(
                                        left.borrow().file_id,
                                        left.borrow().span,
                                    ))
                                    .with_message("the left operand")
                                    .with_label(
                                        Label::primary(right.borrow().file_id, right.borrow().span)
                                            .with_message("the right operand"),
                                    ));
                            }
                        }
                        BinOpKind::Assign => {
                            if left.borrow().is_lvalue && left.borrow().r#type.borrow().can_modify()
                            {
                                let r#type = remove_qualifier(Rc::clone(&left.borrow().r#type));
                                let value = right.borrow().value.clone();

                                match &mut node.kind {
                                    ExprKind::BinOp { left, right, .. } => {
                                        *right = self.try_implicit_cast(
                                            Rc::clone(right),
                                            remove_qualifier(Rc::clone(&left.borrow().r#type)),
                                        )?;
                                    }
                                    _ => unreachable!(),
                                }

                                node.value = value;
                                node.r#type = r#type;
                            } else if !left.borrow().r#type.borrow().is_complete() {
                                return Err(Diagnostic::error()
                                    .with_message(format!(
                                        "'{}' is not complete",
                                        left.borrow().r#type.borrow().to_string()
                                    ))
                                    .with_label(Label::primary(
                                        left.borrow().file_id,
                                        left.borrow().span,
                                    )));
                            } else {
                                return Err(Diagnostic::error().with_message(format!(
                                        "the operator of '{op}' must be a lvalue and can be modified"
                                    )).with_label(Label::primary(left.borrow().file_id,left.borrow().span)).with_message("the left operand")
                                .with_label(Label::primary(right.borrow().file_id,right.borrow().span).with_message("the right operand")));
                            }
                        }
                        BinOpKind::MulAssign
                        | BinOpKind::DivAssign
                        | BinOpKind::ModAssign
                        | BinOpKind::AddAssign
                        | BinOpKind::SubAssign
                        | BinOpKind::LShiftAssign
                        | BinOpKind::RShiftAssign
                        | BinOpKind::BitAndAssign
                        | BinOpKind::BitOrAssign
                        | BinOpKind::BitXOrAssign => {
                            //表达式 lhs @= rhs 与 lhs = lhs @ ( rhs ) 完全相同
                            let eq_expr = Rc::new(RefCell::new(Expr::new(
                                node.file_id,
                                node.span,
                                ExprKind::BinOp {
                                    op: match op {
                                        BinOpKind::MulAssign => BinOpKind::Mul,
                                        BinOpKind::DivAssign => BinOpKind::Div,
                                        BinOpKind::ModAssign => BinOpKind::Mod,
                                        BinOpKind::AddAssign => BinOpKind::Add,
                                        BinOpKind::SubAssign => BinOpKind::Sub,
                                        BinOpKind::LShiftAssign => BinOpKind::LShift,
                                        BinOpKind::RShiftAssign => BinOpKind::RShift,
                                        BinOpKind::BitAndAssign => BinOpKind::BitAnd,
                                        BinOpKind::BitOrAssign => BinOpKind::BitOr,
                                        BinOpKind::BitXOrAssign => BinOpKind::BitXOr,
                                        _ => unreachable!(),
                                    },
                                    left: Rc::clone(left),
                                    right: Rc::clone(right),
                                },
                            )));
                            let r#type = remove_qualifier(Rc::clone(&left.borrow().r#type));

                            self.visit_expr(Rc::clone(&eq_expr))?;
                            match (&eq_expr.borrow().kind, &mut node.kind) {
                                (
                                    ExprKind::BinOp {
                                        right: eq_right, ..
                                    },
                                    ExprKind::BinOp { right, .. },
                                ) => {
                                    *right = Rc::clone(eq_right);
                                }
                                _ => unreachable!(),
                            }

                            node.value = eq_expr.borrow().value.clone();
                            node.r#type = r#type;
                        }
                        BinOpKind::Comma => {
                            self.visit_expr(Rc::clone(left))?;
                            self.visit_expr(Rc::clone(right))?;

                            let r#type = Rc::clone(&left.borrow().r#type);
                            let value = left.borrow().value.clone();

                            node.value = value;
                            node.r#type = r#type;
                        }
                    }
                }
                ExprKind::Conditional {
                    condition,
                    true_expr,
                    false_expr,
                } => {
                    self.visit_expr(Rc::clone(condition))?;
                    self.visit_expr(Rc::clone(true_expr))?;
                    self.visit_expr(Rc::clone(false_expr))?;

                    if true_expr.borrow().r#type.borrow().is_arithmetic()
                        && false_expr.borrow().r#type.borrow().is_arithmetic()
                    {
                        let (a, b) = usual_arith_cast(
                            Rc::clone(&true_expr.borrow().r#type),
                            Rc::clone(&false_expr.borrow().r#type),
                        )?;

                        match &mut node.kind {
                            ExprKind::Conditional {
                                true_expr,
                                false_expr,
                                ..
                            } => {
                                *true_expr =
                                    self.try_implicit_cast(Rc::clone(true_expr), Rc::clone(&a))?;
                                *false_expr =
                                    self.try_implicit_cast(Rc::clone(false_expr), Rc::clone(&b))?;
                            }
                            _ => unreachable!(),
                        }

                        node.r#type = arith_result_type(a, b);
                    }
                    //主要是相同的record类型
                    else if Rc::ptr_eq(&true_expr.borrow().r#type, &false_expr.borrow().r#type) {
                        let r#type = Rc::clone(&true_expr.borrow().r#type);
                        node.r#type = r#type;
                    } else if true_expr.borrow().r#type.borrow().is_void()
                        && false_expr.borrow().r#type.borrow().is_void()
                    {
                        node.r#type = Rc::new(RefCell::new(Type {
                            kind: TypeKind::Void,
                            ..Type::new(node.file_id, node.span)
                        }));
                    } else if true_expr.borrow().r#type.borrow().is_pointer()
                        || true_expr.borrow().r#type.borrow().is_nullptr()
                            && false_expr.borrow().r#type.borrow().is_pointer()
                        || false_expr.borrow().r#type.borrow().is_nullptr()
                    {
                        match &mut node.kind {
                            ExprKind::Conditional {
                                true_expr,
                                false_expr,
                                ..
                            } => {
                                let new_true_expr = self.try_implicit_cast(
                                    Rc::clone(true_expr),
                                    Rc::new(RefCell::new(Type {
                                        kind: TypeKind::Pointer(Rc::new(RefCell::new(Type {
                                            kind: TypeKind::Void,
                                            ..Type::new(
                                                true_expr.borrow().file_id,
                                                true_expr.borrow().span,
                                            )
                                        }))),
                                        ..Type::new(
                                            true_expr.borrow().file_id,
                                            true_expr.borrow().span,
                                        )
                                    })),
                                )?;
                                let new_false_expr = self.try_implicit_cast(
                                    Rc::clone(false_expr),
                                    Rc::new(RefCell::new(Type {
                                        kind: TypeKind::Pointer(Rc::new(RefCell::new(Type {
                                            kind: TypeKind::Void,
                                            ..Type::new(
                                                false_expr.borrow().file_id,
                                                false_expr.borrow().span,
                                            )
                                        }))),
                                        ..Type::new(
                                            false_expr.borrow().file_id,
                                            false_expr.borrow().span,
                                        )
                                    })),
                                )?;
                                *true_expr = new_true_expr;
                                *false_expr = new_false_expr;
                            }
                            _ => unreachable!(),
                        }
                        node.r#type = Rc::new(RefCell::new(Type {
                            kind: TypeKind::Pointer(Rc::new(RefCell::new(Type {
                                kind: TypeKind::Void,
                                ..Type::new(node.file_id, node.span)
                            }))),
                            ..Type::new(node.file_id, node.span)
                        }));
                    } else {
                        return Err(Diagnostic::error().with_message(format!(
                                "the operand of conditional expression must have an arithmetic or void type or is a pointer"
                            )).with_label(Label::primary(true_expr.borrow().file_id,true_expr.borrow().span).with_message("the true-expression")).with_label(Label::primary(false_expr.borrow().file_id,false_expr.borrow().span).with_message("the false-expression")));
                    }
                }
                ExprKind::SizeOf {
                    r#type: Some(_),
                    expr: Some(_),
                    ..
                } => {
                    return Err(Diagnostic::error()
                        .with_message(format!("cannot disambiguate sizeof"))
                        .with_label(Label::primary(node.file_id, node.span)));
                }
                ExprKind::SizeOf {
                    r#type: Some(r#type),
                    expr: None,
                    ..
                } => {
                    let value;
                    match r#type.borrow().size() {
                        Some(t) => value = Variant::Int(BigInt::from(t)),
                        None => {
                            return Err(Diagnostic::error()
                                .with_message(format!(
                                    "'{}' is incomplete",
                                    r#type.borrow().to_string()
                                ))
                                .with_label(Label::primary(
                                    r#type.borrow().file_id,
                                    r#type.borrow().span,
                                )));
                        }
                    }
                    node.value = value;
                    node.r#type = Rc::new(RefCell::new(Type {
                        kind: TypeKind::ULongLong,
                        ..Type::new(node.file_id, node.span)
                    }));
                }
                ExprKind::SizeOf {
                    r#type: None,
                    expr: Some(expr),
                    ..
                } => {
                    self.visit_expr(Rc::clone(expr))?;

                    let value;
                    match expr.borrow().r#type.borrow().size() {
                        Some(t) => value = Variant::Int(BigInt::from(t)),
                        None => {
                            return Err(Diagnostic::error()
                                .with_message(format!(
                                    "'{}' is incomplete",
                                    expr.borrow().r#type.borrow().to_string()
                                ))
                                .with_label(Label::primary(
                                    expr.borrow().file_id,
                                    expr.borrow().span,
                                )));
                        }
                    }
                    node.value = value;
                    node.r#type = Rc::new(RefCell::new(Type {
                        kind: TypeKind::ULongLong,
                        ..Type::new(node.file_id, node.span)
                    }))
                }
                ExprKind::SizeOf {
                    r#type: None,
                    expr: None,
                    ..
                } => {
                    return Err(Diagnostic::error()
                        .with_message(format!("errors occurred during disambiguation"))
                        .with_label(
                            Label::primary(node.file_id, node.span)
                                .with_message("the location where ambiguity occurs"),
                        )
                        .with_labels({
                            let mut labels = Vec::new();

                            for err in errs {
                                for label in err.labels {
                                    if let LabelStyle::Primary = label.style {
                                        labels.push(
                                            Label::primary(label.file_id, label.range)
                                                .with_message(err.message.clone()),
                                        );
                                        break;
                                    }
                                }
                            }

                            labels
                        }));
                }
                ExprKind::Alignof { r#type, .. } => {
                    if r#type.borrow().is_function() {
                        return Err(Diagnostic::error()
                            .with_message(format!("the operand of alignof cannot be a function"))
                            .with_label(Label::primary(node.file_id, node.span)));
                    }
                    if !r#type.borrow().is_complete() {
                        return Err(Diagnostic::error()
                            .with_message(format!("the operand of alignof is not complete"))
                            .with_label(Label::primary(node.file_id, node.span)));
                    }

                    match if r#type.borrow().is_array() {
                        array_element(Rc::clone(r#type)).unwrap().borrow().align()
                    } else {
                        r#type.borrow().align()
                    } {
                        Some(t) => node.value = Variant::Int(BigInt::from(t)),
                        None => {
                            return Err(Diagnostic::error()
                                .with_message(format!(
                                    "cannot get alignof '{}'",
                                    r#type.borrow().to_string()
                                ))
                                .with_label(Label::primary(node.file_id, node.span)));
                        }
                    }

                    node.r#type = Rc::new(RefCell::new(Type {
                        kind: TypeKind::ULongLong,
                        ..Type::new(node.file_id, node.span)
                    }));
                }
                ExprKind::CompoundLiteral {
                    storage_classes,
                    initializer,
                    ..
                } => {
                    for storage_class in storage_classes {
                        match &storage_class.kind {
                            StorageClassKind::Constexpr
                            | StorageClassKind::Static
                            | StorageClassKind::Register
                            | StorageClassKind::ThreadLocal => {}
                            _ => {
                                return Err(Diagnostic::error().with_message(format!(
                                        "only constexpr, static, register, thread_local are allowed in compound literal"
                                    )).with_label(Label::primary(storage_class.file_id,storage_class.span)));
                            }
                        }
                    }
                    self.visit_initializer(Rc::clone(&initializer), Rc::clone(&node.r#type))?;

                    node.is_lvalue = true;
                }
            }
        }

        if node.borrow().r#type.borrow().is_array() {
            if allow_array_to_ptr {
                let expr = self.wrap_implicit_cast(
                    Rc::new(RefCell::new(node.borrow().clone())),
                    array_to_ptr(Rc::clone(&node.borrow().r#type)),
                );
                node.replace(expr);
            }
        } else if allow_lvalue_cast && node.borrow().is_lvalue {
            let expr = self.wrap_implicit_cast(
                Rc::new(RefCell::new(node.borrow().clone())),
                lvalue_cast(Rc::clone(&node.borrow().r#type)),
            );
            node.replace(expr);
        }

        if allow_func_to_ptr && node.borrow().r#type.borrow().is_function() {
            let expr = self.wrap_implicit_cast(
                Rc::new(RefCell::new(node.borrow().clone())),
                func_to_ptr(Rc::clone(&node.borrow().r#type)),
            );
            node.replace(expr);
        }
        self.contexts.pop();
        Ok(())
    }
}
