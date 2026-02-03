use codespan_reporting::diagnostic::{Diagnostic, Label};

use crate::{
    ast::{Attribute, AttributeKind},
    ctype::{Type, TypeKind, is_compatible},
};
use std::{cell::RefCell, rc::Rc};

pub fn remove_qualifier(a: Rc<RefCell<Type>>) -> Rc<RefCell<Type>> {
    match &a.borrow().kind {
        TypeKind::Qualified { r#type, .. } => Rc::clone(r#type),
        _ => Rc::clone(&a),
    }
}

pub fn remove_atomic(a: Rc<RefCell<Type>>) -> Rc<RefCell<Type>> {
    match &a.borrow().kind {
        TypeKind::Atomic(r#type) => Rc::clone(r#type),
        _ => Rc::clone(&a),
    }
}

pub fn array_to_ptr(a: Rc<RefCell<Type>>) -> Rc<RefCell<Type>> {
    match &a.borrow().kind {
        TypeKind::Array { element_type, .. } => Rc::new(RefCell::new(Type {
            attributes: {
                let mut attributes = a.borrow().attributes.clone();
                attributes.push(Rc::new(RefCell::new(Attribute {
                    name: "ptr_from_array".to_string(),
                    kind: AttributeKind::PtrFromArray {
                        array_type: Rc::clone(&a),
                    },
                    ..Attribute::new(a.borrow().file_id, a.borrow().span)
                })));
                attributes
            },
            kind: TypeKind::Pointer(Rc::clone(&element_type)),
            ..a.borrow().clone()
        })),
        _ => Rc::clone(&a),
    }
}

pub fn func_to_ptr(a: Rc<RefCell<Type>>) -> Rc<RefCell<Type>> {
    match &a.borrow().kind {
        TypeKind::Function { .. } => Rc::new(RefCell::new(Type {
            kind: TypeKind::Pointer(Rc::clone(&a)),
            ..a.borrow().clone()
        })),
        _ => Rc::clone(&a),
    }
}

pub fn lvalue_cast(a: Rc<RefCell<Type>>) -> Rc<RefCell<Type>> {
    remove_atomic(remove_qualifier(a))
}

pub fn integer_promote(a: Rc<RefCell<Type>>) -> Rc<RefCell<Type>> {
    match &a.borrow().kind {
        TypeKind::Bool => Rc::new(RefCell::new(Type {
            kind: TypeKind::Int,
            ..a.borrow().clone()
        })),
        TypeKind::Char
        | TypeKind::SignedChar
        | TypeKind::UnsignedChar
        | TypeKind::Short
        | TypeKind::UShort => {
            let (min, max) = a.borrow().kind.range().unwrap();
            let (int_min, int_max) = TypeKind::Int.range().unwrap();
            Rc::new(RefCell::new(Type {
                kind: if int_min <= min && max <= int_max {
                    TypeKind::Int
                } else {
                    TypeKind::UInt
                },
                ..a.borrow().clone()
            }))
        }
        _ => Rc::clone(&a),
    }
}

pub fn usual_arith_cast(
    a: Rc<RefCell<Type>>,
    b: Rc<RefCell<Type>>,
) -> Result<(Rc<RefCell<Type>>, Rc<RefCell<Type>>), Diagnostic<usize>> {
    let decimal_rank = |x: &TypeKind| match x {
        TypeKind::Decimal128 => 3,
        TypeKind::Decimal64 => 2,
        TypeKind::Decimal32 => 1,
        _ => 0,
    };
    match (&a.borrow().kind, &b.borrow().kind) {
        (
            TypeKind::Decimal128 | TypeKind::Decimal64 | TypeKind::Decimal32,
            TypeKind::Float | TypeKind::Double | TypeKind::LongDouble | TypeKind::Complex(..),
        ) => Err(Diagnostic::error()
            .with_message(format!(
                "cannot mix operands of decimal floating and other floating types"
            ))
            .with_label(Label::primary(a.borrow().file_id, a.borrow().span))),
        (
            TypeKind::Float | TypeKind::Double | TypeKind::LongDouble | TypeKind::Complex(..),
            TypeKind::Decimal128 | TypeKind::Decimal64 | TypeKind::Decimal32,
        ) => Err(Diagnostic::error()
            .with_message(format!(
                "cannot mix operands of decimal floating and other floating types"
            ))
            .with_label(Label::primary(b.borrow().file_id, b.borrow().span))),
        (x @ (TypeKind::Decimal128 | TypeKind::Decimal64 | TypeKind::Decimal32), y)
        | (x, y @ (TypeKind::Decimal128 | TypeKind::Decimal64 | TypeKind::Decimal32)) => {
            if decimal_rank(x) > decimal_rank(y) {
                Ok((
                    Rc::clone(&a),
                    Rc::new(RefCell::new(Type {
                        kind: x.clone(),
                        ..b.borrow().clone()
                    })),
                ))
            } else {
                Ok((
                    Rc::new(RefCell::new(Type {
                        kind: y.clone(),
                        ..a.borrow().clone()
                    })),
                    Rc::clone(&b),
                ))
            }
        }
        (x @ (TypeKind::LongDouble | TypeKind::Double | TypeKind::Float), y) => Ok((
            Rc::clone(&a),
            Rc::new(RefCell::new(Type {
                kind: match y {
                    TypeKind::Complex(..) => TypeKind::Complex(Some(Rc::new(RefCell::new(Type {
                        kind: x.clone(),
                        ..b.borrow().clone()
                    })))),
                    _ => x.clone(),
                },
                ..b.borrow().clone()
            })),
        )),
        (x, y @ (TypeKind::LongDouble | TypeKind::Double | TypeKind::Float)) => Ok((
            Rc::new(RefCell::new(Type {
                kind: match x {
                    TypeKind::Complex(..) => TypeKind::Complex(Some(Rc::new(RefCell::new(Type {
                        kind: y.clone(),
                        ..a.borrow().clone()
                    })))),
                    _ => y.clone(),
                },
                ..a.borrow().clone()
            })),
            Rc::clone(&b),
        )),
        (x, y) if x.is_integer() && y.is_integer() => {
            let x = integer_promote(Rc::clone(&a));
            let y = integer_promote(Rc::clone(&b));
            if is_compatible(Rc::clone(&x), Rc::clone(&y)) {
                Ok((x, y))
            } else {
                let (x_min, x_max) = x.borrow().range().unwrap();
                let (y_min, y_max) = y.borrow().range().unwrap();
                if y_min <= x_min && x_max <= y_max {
                    Ok((
                        Rc::new(RefCell::new(Type {
                            kind: y.clone().borrow().kind.clone(),
                            ..x.borrow().clone()
                        })),
                        y,
                    ))
                } else if x_min <= y_min && y_max <= x_max {
                    Ok((
                        x.clone(),
                        Rc::new(RefCell::new(Type {
                            kind: x.borrow().kind.clone(),
                            ..y.borrow().clone()
                        })),
                    ))
                }
                // 从这之后x, y一个有符号一个没符号
                else if let Some(false) = x.borrow().is_unsigned() {
                    Ok((
                        Rc::new(RefCell::new(Type {
                            kind: x.borrow().kind.to_unsigned().unwrap(),
                            ..x.borrow().clone()
                        })),
                        Rc::new(RefCell::new(Type {
                            kind: x.borrow().kind.to_unsigned().unwrap(),
                            ..y.borrow().clone()
                        })),
                    ))
                } else if let Some(false) = y.borrow().is_unsigned() {
                    Ok((
                        Rc::new(RefCell::new(Type {
                            kind: y.borrow().kind.to_unsigned().unwrap(),
                            ..x.borrow().clone()
                        })),
                        Rc::new(RefCell::new(Type {
                            kind: y.borrow().kind.to_unsigned().unwrap(),
                            ..y.borrow().clone()
                        })),
                    ))
                } else {
                    Ok((x, y))
                }
            }
        }
        _ => Ok((Rc::clone(&a), Rc::clone(&b))),
    }
}
