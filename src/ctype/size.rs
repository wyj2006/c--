use crate::{
    ctype::{RecordKind, Type, TypeKind},
    variant::Variant,
};
use num::ToPrimitive;

impl Type {
    pub fn size(&self) -> Option<usize> {
        self.kind.size()
    }
}

impl TypeKind {
    pub fn size(&self) -> Option<usize> {
        match &self {
            //TODO 根据平台决定
            TypeKind::Char | TypeKind::SignedChar | TypeKind::UnsignedChar => Some(1),
            TypeKind::Short | TypeKind::UShort => Some(2),
            TypeKind::Unsigned | TypeKind::Signed | TypeKind::Int | TypeKind::UInt => Some(4),
            TypeKind::Long | TypeKind::ULong => Some(4),
            TypeKind::LongLong | TypeKind::ULongLong => Some(8),
            TypeKind::Pointer(_) | TypeKind::Nullptr => Some(8),
            TypeKind::BitInt { width_expr, .. } => {
                if let Variant::Int(n) = &width_expr.borrow().value {
                    Some(n.to_usize()?.div_ceil(8))
                } else {
                    None
                }
            }
            TypeKind::Bool => Some(1),
            TypeKind::Float => Some(4),
            TypeKind::Double => Some(8),
            TypeKind::LongDouble => Some(16),
            TypeKind::Complex(Some(t)) => match t.borrow().size() {
                Some(t) => Some(t * 2),
                None => None,
            },
            TypeKind::Decimal32 => Some(4),
            TypeKind::Decimal64 => Some(8),
            TypeKind::Decimal128 => Some(16),
            TypeKind::Enum { underlying, .. } => underlying.borrow().size(),
            TypeKind::Auto(Some(t))
            | TypeKind::Atomic(t)
            | TypeKind::Qualified { r#type: t, .. }
            | TypeKind::Typedef {
                r#type: Some(t), ..
            } => t.borrow().size(),
            TypeKind::Typeof {
                expr: Some(t),
                r#type: None,
                ..
            } => t.borrow().r#type.borrow().size(),
            TypeKind::Typeof {
                expr: None,
                r#type: Some(t),
                ..
            } => t.borrow().size(),
            TypeKind::Array {
                element_type,
                len_expr: Some(len_expr),
                ..
            } => match &len_expr.borrow().value {
                Variant::Int(value) => match (
                    value.to_usize(),
                    element_type.borrow().size(),
                    element_type.borrow().align(),
                ) {
                    (Some(len), Some(element_size), Some(element_align)) => {
                        Some(len * element_size.div_ceil(element_align) * element_align)
                    }
                    _ => None,
                },
                _ => None,
            },
            TypeKind::Record {
                kind: RecordKind::Union,
                members: Some(members),
                ..
            } => {
                let mut size = 0;
                for (_, member) in members {
                    size = size.max(member.borrow().r#type.borrow().size()?);
                }
                Some(size)
            }
            TypeKind::Record {
                kind: RecordKind::Struct,
                members: Some(members),
                ..
            } => {
                let mut size: usize = 0;
                for (_, member) in members {
                    let member_align = member.borrow().r#type.borrow().align()?;
                    size = size.div_ceil(member_align) * member_align;
                    size += member.borrow().r#type.borrow().size()?;
                }
                Some(size)
            }
            _ => None,
        }
    }
}
