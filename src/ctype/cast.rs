use crate::{
    ast::{Attribute, AttributeKind},
    ctype::{Type, TypeKind, is_compatible},
    diagnostic::{Error, ErrorKind},
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
            span: a.borrow().span,
            attributes: {
                let mut attributes = a.borrow().attributes.clone();
                attributes.push(Rc::new(RefCell::new(Attribute {
                    span: a.borrow().span,
                    prefix_name: None,
                    name: "ptr_from_array".to_string(),
                    kind: AttributeKind::PtrFromArray {
                        array_type: Rc::clone(&a),
                    },
                })));
                attributes
            },
            kind: TypeKind::Pointer(Rc::clone(&element_type)),
        })),
        _ => Rc::clone(&a),
    }
}

pub fn func_to_ptr(a: Rc<RefCell<Type>>) -> Rc<RefCell<Type>> {
    match &a.borrow().kind {
        TypeKind::Function { .. } => Rc::new(RefCell::new(Type {
            span: a.borrow().span,
            attributes: a.borrow().attributes.clone(),
            kind: TypeKind::Pointer(Rc::clone(&a)),
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
            span: a.borrow().span,
            attributes: a.borrow().attributes.clone(),
            kind: TypeKind::Int,
        })),
        TypeKind::Char
        | TypeKind::SignedChar
        | TypeKind::UnsignedChar
        | TypeKind::Short
        | TypeKind::UShort => {
            let (min, max) = a.borrow().kind.range().unwrap();
            let (int_min, int_max) = TypeKind::Int.range().unwrap();
            Rc::new(RefCell::new(Type {
                span: a.borrow().span,
                attributes: a.borrow().attributes.clone(),
                kind: if int_min <= min && max <= int_max {
                    TypeKind::Int
                } else {
                    TypeKind::UInt
                },
            }))
        }
        _ => Rc::clone(&a),
    }
}

pub fn usual_arith_cast<'a>(
    a: Rc<RefCell<Type<'a>>>,
    b: Rc<RefCell<Type<'a>>>,
) -> Result<(Rc<RefCell<Type<'a>>>, Rc<RefCell<Type<'a>>>), Error<'a>> {
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
        ) => Err(Error {
            span: a.borrow().span,
            kind: ErrorKind::Other(format!(
                "cannot mix operands of decimal floating and other floating types"
            )),
        }),
        (
            TypeKind::Float | TypeKind::Double | TypeKind::LongDouble | TypeKind::Complex(..),
            TypeKind::Decimal128 | TypeKind::Decimal64 | TypeKind::Decimal32,
        ) => Err(Error {
            span: b.borrow().span,
            kind: ErrorKind::Other(format!(
                "cannot mix operands of decimal floating and other floating types"
            )),
        }),
        (x @ (TypeKind::Decimal128 | TypeKind::Decimal64 | TypeKind::Decimal32), y)
        | (x, y @ (TypeKind::Decimal128 | TypeKind::Decimal64 | TypeKind::Decimal32)) => {
            if decimal_rank(x) > decimal_rank(y) {
                Ok((
                    Rc::clone(&a),
                    Rc::new(RefCell::new(Type {
                        span: b.borrow().span,
                        attributes: b.borrow().attributes.clone(),
                        kind: x.clone(),
                    })),
                ))
            } else {
                Ok((
                    Rc::new(RefCell::new(Type {
                        span: a.borrow().span,
                        attributes: a.borrow().attributes.clone(),
                        kind: y.clone(),
                    })),
                    Rc::clone(&b),
                ))
            }
        }
        (x @ (TypeKind::LongDouble | TypeKind::Double | TypeKind::Float), y) => Ok((
            Rc::clone(&a),
            Rc::new(RefCell::new(Type {
                span: b.borrow().span,
                attributes: b.borrow().attributes.clone(),
                kind: match y {
                    TypeKind::Complex(..) => TypeKind::Complex(Some(Rc::new(RefCell::new(Type {
                        span: b.borrow().span,
                        attributes: b.borrow().attributes.clone(),
                        kind: x.clone(),
                    })))),
                    _ => x.clone(),
                },
            })),
        )),
        (x, y @ (TypeKind::LongDouble | TypeKind::Double | TypeKind::Float)) => Ok((
            Rc::new(RefCell::new(Type {
                span: a.borrow().span,
                attributes: a.borrow().attributes.clone(),
                kind: match x {
                    TypeKind::Complex(..) => TypeKind::Complex(Some(Rc::new(RefCell::new(Type {
                        span: a.borrow().span,
                        attributes: a.borrow().attributes.clone(),
                        kind: y.clone(),
                    })))),
                    _ => y.clone(),
                },
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
                            span: x.borrow().span,
                            attributes: x.borrow().attributes.clone(),
                            kind: y.clone().borrow().kind.clone(),
                        })),
                        y,
                    ))
                } else if x_min <= y_min && y_max <= x_max {
                    Ok((
                        x.clone(),
                        Rc::new(RefCell::new(Type {
                            span: y.borrow().span,
                            attributes: y.borrow().attributes.clone(),
                            kind: x.borrow().kind.clone(),
                        })),
                    ))
                }
                // 从这之后x, y一个有符号一个没符号
                else if let Some(false) = x.borrow().is_unsigned() {
                    Ok((
                        Rc::new(RefCell::new(Type {
                            span: x.borrow().span,
                            attributes: x.borrow().attributes.clone(),
                            kind: x.borrow().kind.to_unsigned().unwrap(),
                        })),
                        Rc::new(RefCell::new(Type {
                            span: y.borrow().span,
                            attributes: y.borrow().attributes.clone(),
                            kind: x.borrow().kind.to_unsigned().unwrap(),
                        })),
                    ))
                } else if let Some(false) = y.borrow().is_unsigned() {
                    Ok((
                        Rc::new(RefCell::new(Type {
                            span: x.borrow().span,
                            attributes: x.borrow().attributes.clone(),
                            kind: y.borrow().kind.to_unsigned().unwrap(),
                        })),
                        Rc::new(RefCell::new(Type {
                            span: y.borrow().span,
                            attributes: y.borrow().attributes.clone(),
                            kind: y.borrow().kind.to_unsigned().unwrap(),
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
