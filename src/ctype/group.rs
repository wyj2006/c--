use crate::ctype::{RecordKind, Type, TypeKind, TypeQual};
use crate::variant::Variant;
use num::BigInt;
use num::pow::Pow;
use std::rc::Rc;

impl Type {
    pub fn is_integer(&self) -> bool {
        self.kind.is_integer()
    }

    pub fn range(&self) -> Option<(BigInt, BigInt)> {
        self.kind.range()
    }

    pub fn is_unsigned(&self) -> Option<bool> {
        self.kind.is_unsigned()
    }

    pub fn is_pointer(&self) -> bool {
        self.kind.is_pointer()
    }

    pub fn is_array(&self) -> bool {
        self.kind.is_array()
    }

    pub fn is_function(&self) -> bool {
        self.kind.is_function()
    }

    pub fn is_real_float(&self) -> bool {
        self.kind.is_real_float()
    }

    pub fn is_complex(&self) -> bool {
        self.kind.is_complex()
    }

    pub fn is_void(&self) -> bool {
        self.kind.is_void()
    }

    pub fn is_void_ptr(&self) -> bool {
        self.kind.is_void_ptr()
    }

    pub fn is_float(&self) -> bool {
        self.kind.is_float()
    }

    pub fn is_arithmetic(&self) -> bool {
        self.kind.is_arithmetic()
    }

    pub fn is_scale(&self) -> bool {
        self.kind.is_scale()
    }

    pub fn is_real(&self) -> bool {
        self.kind.is_real()
    }

    pub fn is_nullptr(&self) -> bool {
        self.kind.is_nullptr()
    }

    pub fn is_record(&self) -> bool {
        self.kind.is_record()
    }

    pub fn is_struct(&self) -> bool {
        self.kind.is_struct()
    }

    pub fn is_union(&self) -> bool {
        self.kind.is_union()
    }

    pub fn is_object(&self) -> bool {
        self.kind.is_object()
    }

    pub fn is_char(&self) -> bool {
        self.kind.is_char()
    }

    pub fn is_const(&self) -> bool {
        self.kind.is_const()
    }

    pub fn is_vla(&self) -> bool {
        self.kind.is_vla()
    }

    pub fn is_bool(&self) -> bool {
        self.kind.is_bool()
    }
}

impl TypeKind {
    pub fn is_integer(&self) -> bool {
        match self {
            TypeKind::Bool
            | TypeKind::Char
            | TypeKind::SignedChar
            | TypeKind::UnsignedChar
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
            | TypeKind::SignedChar
            | TypeKind::Signed
            | TypeKind::Short
            | TypeKind::Int
            | TypeKind::Long
            | TypeKind::LongLong => Some(false),
            TypeKind::Bool
            | TypeKind::UnsignedChar
            | TypeKind::Unsigned
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
            | TypeKind::SignedChar
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
            TypeKind::Bool
            | TypeKind::UnsignedChar
            | TypeKind::Unsigned
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

    pub fn to_unsigned(&self) -> Option<TypeKind> {
        match self {
            TypeKind::Bool
            | TypeKind::Unsigned
            | TypeKind::UShort
            | TypeKind::UInt
            | TypeKind::ULong
            | TypeKind::ULongLong => Some(self.clone()),
            TypeKind::Char | TypeKind::SignedChar => Some(TypeKind::UnsignedChar),
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

    pub fn is_pointer(&self) -> bool {
        match self {
            TypeKind::Pointer(_) => true,
            TypeKind::Qualified { r#type, .. } => r#type.borrow().is_pointer(),
            _ => false,
        }
    }

    pub fn is_array(&self) -> bool {
        match self {
            TypeKind::Array { .. } => true,
            TypeKind::Qualified { r#type, .. } => r#type.borrow().is_array(),
            _ => false,
        }
    }

    pub fn is_function(&self) -> bool {
        match self {
            TypeKind::Function { .. } => true,
            TypeKind::Qualified { r#type, .. } => r#type.borrow().is_function(),
            _ => false,
        }
    }

    pub fn is_real_float(&self) -> bool {
        match self {
            TypeKind::Float | TypeKind::Double | TypeKind::LongDouble => true,
            TypeKind::Qualified { r#type, .. } => r#type.borrow().is_real_float(),
            _ => false,
        }
    }

    pub fn is_complex(&self) -> bool {
        match self {
            TypeKind::Complex(..) => true,
            TypeKind::Qualified { r#type, .. } => r#type.borrow().is_complex(),
            _ => false,
        }
    }

    pub fn is_void(&self) -> bool {
        match self {
            TypeKind::Void => true,
            _ => false,
        }
    }

    pub fn is_void_ptr(&self) -> bool {
        match self {
            TypeKind::Pointer(pointee) => pointee.borrow().is_void(),
            TypeKind::Qualified { r#type, .. } => r#type.borrow().is_void_ptr(),
            _ => false,
        }
    }

    pub fn is_float(&self) -> bool {
        match self {
            TypeKind::Decimal32
            | TypeKind::Decimal64
            | TypeKind::Decimal128
            | TypeKind::Complex(..) => true,
            TypeKind::Qualified { r#type, .. } => r#type.borrow().is_float(),
            _ => self.is_real_float(),
        }
    }

    pub fn is_arithmetic(&self) -> bool {
        self.is_integer() || self.is_float()
    }

    pub fn is_scale(&self) -> bool {
        match self {
            TypeKind::Nullptr => true,
            _ => self.is_arithmetic() || self.is_pointer(),
        }
    }

    pub fn is_real(&self) -> bool {
        self.is_integer() || self.is_real_float()
    }

    pub fn is_nullptr(&self) -> bool {
        match self {
            TypeKind::Nullptr => true,
            _ => false,
        }
    }

    pub fn is_record(&self) -> bool {
        self.is_struct() || self.is_union()
    }

    pub fn is_struct(&self) -> bool {
        match self {
            TypeKind::Record {
                kind: RecordKind::Struct,
                ..
            } => true,
            TypeKind::Qualified { r#type, .. } => r#type.borrow().is_struct(),
            _ => false,
        }
    }

    pub fn is_union(&self) -> bool {
        match self {
            TypeKind::Record {
                kind: RecordKind::Union,
                ..
            } => true,
            TypeKind::Qualified { r#type, .. } => r#type.borrow().is_union(),
            _ => false,
        }
    }

    pub fn is_object(&self) -> bool {
        match self {
            TypeKind::Function { .. } => false,
            _ => true,
        }
    }

    pub fn is_char(&self) -> bool {
        match self {
            TypeKind::Char | TypeKind::UnsignedChar | TypeKind::SignedChar => true,
            _ => false,
        }
    }

    pub fn is_const(&self) -> bool {
        match self {
            TypeKind::Qualified { qualifiers, .. } => qualifiers.contains(&TypeQual::Const),
            _ => false,
        }
    }

    pub fn is_vla(&self) -> bool {
        match self {
            TypeKind::Array {
                len_expr: Some(len_expr),
                ..
            } => matches!(len_expr.borrow().value, Variant::Unknown),
            TypeKind::Qualified { r#type, .. } => r#type.borrow().is_vla(),
            _ => false,
        }
    }

    pub fn is_bool(&self) -> bool {
        match self {
            TypeKind::Bool => true,
            TypeKind::Qualified { r#type, .. } => r#type.borrow().is_bool(),
            _ => false,
        }
    }
}
