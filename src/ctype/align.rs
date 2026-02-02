use crate::{
    ast::AttributeKind,
    ctype::{Type, TypeKind},
    variant::Variant,
};
use num::ToPrimitive;

impl<'a> Type<'a> {
    pub fn has_alignas(&self) -> bool {
        for attribute in &self.attributes {
            match &attribute.borrow().kind {
                AttributeKind::AlignAs { .. } => return true,
                _ => {}
            }
        }
        false
    }

    pub fn align(&self) -> Option<usize> {
        let mut align = self.kind.align();
        for attribute in &self.attributes {
            match &attribute.borrow().kind {
                AttributeKind::AlignAs {
                    r#type: Some(r#type),
                    expr: None,
                } => {
                    align = match (r#type.borrow().align(), align) {
                        (Some(a), Some(b)) => Some(a.max(b)),
                        (Some(a), None) | (None, Some(a)) => Some(a),
                        (None, None) => align,
                    }
                }
                AttributeKind::AlignAs {
                    r#type: None,
                    expr: Some(expr),
                } => match &expr.borrow().value {
                    Variant::Int(value) => {
                        align = match (value.to_usize(), align) {
                            (Some(a), Some(b)) => Some(a.max(b)),
                            (Some(a), None) | (None, Some(a)) => Some(a),
                            (None, None) => align,
                        }
                    }
                    //忽略错误, 在其它地方检查
                    _ => {}
                },
                _ => {}
            }
        }
        align
    }
}

impl<'a> TypeKind<'a> {
    pub fn align(&self) -> Option<usize> {
        match self {
            //TODO 根据平台设置
            TypeKind::Char | TypeKind::SignedChar | TypeKind::UnsignedChar => Some(1),
            TypeKind::Short | TypeKind::UShort => Some(2),
            TypeKind::Unsigned | TypeKind::Signed | TypeKind::Int | TypeKind::UInt => Some(4),
            TypeKind::Long | TypeKind::ULong => Some(4),
            TypeKind::LongLong | TypeKind::ULongLong => Some(8),
            TypeKind::Pointer(_) | TypeKind::Nullptr => Some(8),
            TypeKind::BitInt { width_expr, .. } => {
                if let Variant::Int(n) = &width_expr.borrow().value {
                    Some(1 << (usize::BITS - n.to_usize()?.leading_zeros() - 1))
                } else {
                    None
                }
            }
            TypeKind::Bool => Some(1),
            TypeKind::Float => Some(4),
            TypeKind::Double => Some(8),
            TypeKind::LongDouble => Some(16),
            TypeKind::Complex(Some(t)) => t.borrow().align(),
            TypeKind::Decimal32 => Some(4),
            TypeKind::Decimal64 => Some(8),
            TypeKind::Decimal128 => Some(16),
            TypeKind::Enum { underlying, .. } => underlying.borrow().align(),
            TypeKind::Array { element_type, .. } => element_type.borrow().align(),
            TypeKind::Auto(Some(t))
            | TypeKind::Atomic(t)
            | TypeKind::Qualified { r#type: t, .. }
            | TypeKind::Typedef {
                r#type: Some(t), ..
            } => t.borrow().align(),
            TypeKind::Typeof {
                expr: Some(t),
                r#type: None,
                ..
            } => t.borrow().r#type.borrow().align(),
            TypeKind::Typeof {
                expr: None,
                r#type: Some(t),
                ..
            } => t.borrow().align(),
            TypeKind::Record {
                members: Some(members),
                ..
            } => {
                let mut align = 0;
                for (_, member) in members {
                    align = align.max(member.borrow().r#type.borrow().align()?);
                }
                Some(align)
            }
            _ => None,
        }
    }
}
