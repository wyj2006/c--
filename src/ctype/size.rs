use std::{cell::RefCell, rc::Rc};

use crate::{
    ast::expr::Expr,
    ctype::{RecordKind, Type, TypeKind},
    symtab::SymbolKind,
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
                    let member_type = &member.borrow().r#type;
                    let bit_field = if let SymbolKind::Member {
                        bit_field: Some(bit_field),
                        ..
                    } = member.borrow().kind
                    {
                        bit_field
                    } else {
                        0
                    };
                    if bit_field > 0 {
                        size = size.max(bit_field.div_ceil(8));
                    }
                    //无名位域
                    if member.borrow().name.len() == 0 {
                        continue;
                    }
                    //处理当前成员
                    size = size.max(member_type.borrow().size()?);
                }
                Some(size)
            }
            TypeKind::Record {
                kind: RecordKind::Struct,
                members: Some(members),
                ..
            } => {
                let mut offset: usize = 0;
                let mut bit_fields = vec![];

                for (i, (_, member)) in members.iter().enumerate() {
                    let member_type = &member.borrow().r#type;
                    let bit_field = if let SymbolKind::Member {
                        bit_field: Some(bit_field),
                        ..
                    } = member.borrow().kind
                    {
                        bit_field
                    } else {
                        0
                    };
                    if bit_field > 0 {
                        bit_fields.push(bit_field);
                        if i < members.len() - 1 {
                            continue;
                        }
                    }
                    if bit_fields.len() > 0 {
                        //处理位域
                        let bitfield_type = Type {
                            kind: TypeKind::BitInt {
                                unsigned: true,
                                width_expr: Rc::new(RefCell::new(Expr::new_const_int(
                                    member_type.borrow().file_id,
                                    member_type.borrow().span,
                                    bit_fields.iter().sum::<usize>(),
                                    Rc::new(RefCell::new(Type {
                                        kind: TypeKind::LongLong,
                                        ..Type::new(
                                            member_type.borrow().file_id,
                                            member_type.borrow().span,
                                        )
                                    })),
                                ))),
                            },
                            ..Type::new(member_type.borrow().file_id, member_type.borrow().span)
                        };

                        let align = bitfield_type.align()?;
                        offset = offset.div_ceil(align) * align;
                        offset += bitfield_type.size()?;

                        bit_fields = vec![];

                        //最后一个成员是位域
                        if i == members.len() - 1 && bit_field > 0 {
                            continue;
                        }
                        //无名位域
                        if member.borrow().name.len() == 0 {
                            continue;
                        }
                    }
                    //处理当前成员
                    let member_align = member_type.borrow().align()?;
                    offset = offset.div_ceil(member_align) * member_align;
                    offset += member.borrow().r#type.borrow().size()?;
                }
                Some(offset)
            }
            _ => None,
        }
    }
}
