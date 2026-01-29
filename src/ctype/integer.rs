use std::rc::Rc;

use crate::ctype::{Type, TypeKind};
use num::BigInt;
use num::pow::Pow;

impl Type<'_> {
    pub fn is_integer(&self) -> bool {
        self.kind.is_integer()
    }

    pub fn range(&self) -> Option<(BigInt, BigInt)> {
        self.kind.range()
    }

    pub fn is_unsigned(&self) -> Option<bool> {
        self.kind.is_unsigned()
    }
}

impl<'a> TypeKind<'a> {
    pub fn is_integer(&self) -> bool {
        match self {
            TypeKind::Char
            | TypeKind::Unsigned
            | TypeKind::Signed
            | TypeKind::Short
            | TypeKind::UShort
            | TypeKind::Int
            | TypeKind::UInt
            | TypeKind::Long
            | TypeKind::ULong
            | TypeKind::LongLong
            | TypeKind::ULongLong
            | TypeKind::BitInt { .. }
            | TypeKind::Enum { .. } => true,
            _ => false,
        }
    }

    pub fn is_unsigned(&self) -> Option<bool> {
        match self {
            TypeKind::Char
            | TypeKind::Signed
            | TypeKind::Short
            | TypeKind::Int
            | TypeKind::Long
            | TypeKind::LongLong => Some(false),
            TypeKind::Unsigned
            | TypeKind::UShort
            | TypeKind::UInt
            | TypeKind::ULong
            | TypeKind::ULongLong => Some(true),
            TypeKind::BitInt { unsigned, .. } => Some(*unsigned),
            TypeKind::Enum { underlying, .. } => underlying.borrow().is_unsigned(),
            _ => None,
        }
    }

    pub fn range(&self) -> Option<(BigInt, BigInt)> {
        let Some(size) = self.size() else {
            return None;
        };
        let size = size * 8;
        match &self {
            TypeKind::Char
            | TypeKind::Signed
            | TypeKind::Short
            | TypeKind::Int
            | TypeKind::Long
            | TypeKind::LongLong
            | TypeKind::BitInt {
                unsigned: false, ..
            } => Some((
                -BigInt::from(2).pow(size - 1),
                BigInt::from(2).pow(size - 1) - 1,
            )),
            TypeKind::Unsigned
            | TypeKind::UShort
            | TypeKind::UInt
            | TypeKind::ULong
            | TypeKind::ULongLong
            | TypeKind::BitInt { unsigned: true, .. } => {
                Some((BigInt::ZERO, BigInt::from(2).pow(size) - 1))
            }
            TypeKind::Enum { underlying, .. } => underlying.borrow().range(),
            _ => None,
        }
    }

    pub fn to_unsigned(&self) -> Option<TypeKind<'a>> {
        match self {
            TypeKind::Unsigned
            | TypeKind::UShort
            | TypeKind::UInt
            | TypeKind::ULong
            | TypeKind::ULongLong => Some(self.clone()),
            TypeKind::Char => Some(TypeKind::UnsignedChar),
            TypeKind::Signed | TypeKind::Int => Some(TypeKind::UInt),
            TypeKind::Short => Some(TypeKind::UShort),
            TypeKind::Long => Some(TypeKind::ULong),
            TypeKind::LongLong => Some(TypeKind::ULongLong),
            TypeKind::BitInt { width_expr, .. } => Some(TypeKind::BitInt {
                unsigned: true,
                width_expr: Rc::clone(width_expr),
            }),
            TypeKind::Enum { underlying, .. } => underlying.borrow().kind.to_unsigned(),
            _ => None,
        }
    }
}
