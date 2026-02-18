use crate::{
    ast::expr::Expr,
    ctype::{RecordKind, Type, TypeKind},
    symtab::SymbolKind,
    variant::Variant,
};
use num::ToPrimitive;
use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Clone)]
pub struct Layout {
    pub r#type: Rc<RefCell<Type>>,
    pub width: usize,
    pub offset: usize, //相对父Layout的偏移
    pub children: Vec<Layout>,
    //相当于Designation的两种类型
    pub index: Option<usize>,
    pub name: Option<String>,
}

pub fn compute_layout(r#type: Rc<RefCell<Type>>) -> Option<Layout> {
    match &r#type.borrow().kind {
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
                    let d_offset = element_size.div_ceil(element_align) * element_align;
                    let mut children = vec![];
                    for i in 0..len {
                        children.push(Layout {
                            offset: i * d_offset,
                            index: Some(i),
                            ..compute_layout(Rc::clone(element_type))?
                        });
                    }
                    Some(Layout {
                        r#type: Rc::clone(&r#type),
                        width: r#type.borrow().size()?,
                        offset: 0,
                        children,
                        index: None,
                        name: None,
                    })
                }
                _ => None,
            },
            _ => None,
        },
        TypeKind::Array { len_expr: None, .. } => None,
        TypeKind::Record {
            kind: RecordKind::Struct,
            members: Some(members),
            ..
        } => {
            let mut children = vec![];
            let mut offset: usize = 0;
            let mut bitfield_children = vec![];
            let mut bitfield_offset = 0;

            for (i, (_, member)) in members.iter().enumerate() {
                let bit_field = match &mut member.borrow_mut().kind {
                    SymbolKind::Member { bit_field, index } => {
                        *index = children.len();
                        bit_field.unwrap_or(0)
                    }
                    _ => unreachable!(),
                };
                let member_type = &member.borrow().r#type;

                if bit_field > 0 {
                    bitfield_children.push(Layout {
                        width: bit_field,
                        offset: bitfield_offset,
                        name: Some(member.borrow().name.clone()),
                        ..compute_layout(Rc::clone(&member_type))?
                    });
                    bitfield_offset += bit_field;
                    if i < members.len() - 1 {
                        continue;
                    }
                }
                if bitfield_children.len() > 0 {
                    //处理位域
                    let bitfield_type = Type {
                        kind: TypeKind::BitInt {
                            unsigned: true,
                            width_expr: Rc::new(RefCell::new(Expr::new_const_int(
                                member_type.borrow().file_id,
                                member_type.borrow().span,
                                bitfield_offset,
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
                    children.push(Layout {
                        children: bitfield_children.clone(),
                        offset,
                        ..compute_layout(Rc::new(RefCell::new(bitfield_type.clone())))?
                    });
                    offset += bitfield_type.size()?;

                    bitfield_children = vec![];
                    bitfield_offset = 0;

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

                children.push(Layout {
                    offset,
                    ..compute_layout(Rc::clone(member_type))?
                });

                offset += member.borrow().r#type.borrow().size()?;
            }
            Some(Layout {
                r#type: Rc::clone(&r#type),
                width: r#type.borrow().size()?,
                offset: 0,
                children,
                index: None,
                name: None,
            })
        }
        TypeKind::Record {
            kind: RecordKind::Union,
            members: Some(members),
            ..
        } => {
            let mut children = vec![];

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
                    children.push(Layout {
                        width: bit_field,
                        name: Some(member.borrow().name.clone()),
                        ..compute_layout(Rc::clone(&member_type))?
                    });
                }
                //无名位域
                if member.borrow().name.len() == 0 {
                    continue;
                }
                //处理当前成员
                children.push(compute_layout(Rc::clone(member_type))?);
            }
            Some(Layout {
                r#type: Rc::clone(&r#type),
                width: r#type.borrow().size()?,
                offset: 0,
                children,
                index: None,
                name: None,
            })
        }
        TypeKind::Record { members: None, .. } => None,
        _ => Some(Layout {
            r#type: Rc::clone(&r#type),
            width: r#type.borrow().size()?,
            offset: 0,
            children: vec![],
            index: None,
            name: None,
        }),
    }
}
