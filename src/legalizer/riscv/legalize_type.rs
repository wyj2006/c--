use crate::{
    ast::{Attribute, AttributeKind},
    ctype::{RecordKind, Type, TypeKind, get_inner_type},
    legalizer::riscv::Legalizer,
    symtab::{Symbol, SymbolKind},
    variant::Variant,
};
use codespan_reporting::diagnostic::Diagnostic;
use indexmap::IndexMap;
use num::ToPrimitive;
use std::{cell::RefCell, rc::Rc};

impl Legalizer {
    pub fn legalize_type(&mut self, r#type: &Rc<RefCell<Type>>) -> Result<(), Diagnostic<usize>> {
        let origin_type = Rc::new(RefCell::new((*r#type.borrow()).clone()));
        let file_id = r#type.borrow().file_id;
        let span = r#type.borrow().span;

        let r#type = get_inner_type(r#type.clone());
        let mut part_type = None;
        let mut bitint = None;

        match &r#type.borrow().kind {
            TypeKind::Array { element_type, .. } => {
                self.legalize_type(element_type)?;
            }
            TypeKind::Function {
                return_type,
                parameters_type,
                ..
            } => {
                self.legalize_type(return_type)?;
                for param_type in parameters_type {
                    self.legalize_type(param_type)?;
                }
            }
            TypeKind::Pointer(r#type) => self.legalize_type(r#type)?,
            TypeKind::Record {
                members: Some(members),
                ..
            } => {
                for (_, member) in members {
                    self.legalize_type(&member.borrow().r#type)?;
                }
            }
            TypeKind::Complex(Some(part_ty)) => part_type = Some(part_ty.clone()),
            TypeKind::BitInt {
                unsigned,
                width_expr,
            } => match &width_expr.borrow().value {
                //直接用 .size 得到的是四舍五入后的以byte为单位的大小
                Variant::Int(a) => bitint = Some((*unsigned, a.to_usize().unwrap_or(0))),
                _ => {}
            },
            //将长度比较大的标准整数类型也当成bitint
            t if t.is_integer() => match t.size() {
                None | Some(1) | Some(2) | Some(4) => {}
                Some(8) if self.xlen == 64 => {}
                Some(size) => bitint = Some((t.is_unsigned().unwrap_or(false), size * 8)),
            },
            _ => {}
        }

        if let Some(part_type) = part_type {
            let mut members = IndexMap::new();

            let real_symbol = Symbol {
                define_loc: Some((file_id, span)),
                declare_locs: vec![(file_id, span)],
                name: "real".to_string(),
                kind: SymbolKind::Member {
                    bit_field: None,
                    index: 0,
                    belong_record: r#type.clone(),
                },
                r#type: part_type.clone(),
                attributes: vec![],
            };
            members.insert("real".to_string(), Rc::new(RefCell::new(real_symbol)));

            let imag_symbol = Symbol {
                define_loc: Some((file_id, span)),
                declare_locs: vec![(file_id, span)],
                name: "imag".to_string(),
                kind: SymbolKind::Member {
                    bit_field: None,
                    index: 1,
                    belong_record: r#type.clone(),
                },
                r#type: part_type.clone(),
                attributes: vec![],
            };
            members.insert("imag".to_string(), Rc::new(RefCell::new(imag_symbol)));

            r#type.borrow_mut().kind = TypeKind::Record {
                name: format!(
                    "__{}Complex",
                    if part_type.borrow().is_float_type() {
                        "Float"
                    } else {
                        "Double"
                    }
                ),
                kind: RecordKind::Struct,
                members: Some(members),
            };
            r#type
                .borrow_mut()
                .attributes
                .push(Rc::new(RefCell::new(Attribute {
                    file_id,
                    span,
                    prefix_name: None,
                    name: "type_before_legalize".to_string(),
                    kind: AttributeKind::TypeBeforeLegalize {
                        origin_type: origin_type.clone(),
                    },
                })));
        }

        //这里的size以bit为单位
        if let Some((unsigned, mut size)) = bitint {
            let origin_size = size;
            let mut members = IndexMap::new();

            let mut i = 0;
            while size > 0 {
                let name = format!("w{i}");

                let symbol = Symbol {
                    define_loc: Some((file_id, span)),
                    declare_locs: vec![(file_id, span)],
                    name: name.clone(),
                    kind: SymbolKind::Member {
                        bit_field: if size >= self.xlen { None } else { Some(size) },
                        index: i,
                        belong_record: r#type.clone(),
                    },
                    r#type: Rc::new(RefCell::new(Type {
                        file_id,
                        span,
                        attributes: vec![],
                        //TODO 根据平台决定
                        kind: TypeKind::ULong,
                    })),
                    attributes: vec![],
                };

                members.insert(name, Rc::new(RefCell::new(symbol)));

                if size < self.xlen {
                    break;
                }
                size -= self.xlen;

                i += 1;
            }

            r#type.borrow_mut().kind = TypeKind::Record {
                name: format!(
                    "__{}BitInt{}",
                    if unsigned { "Unsigned" } else { "Signed" },
                    origin_size
                ),
                kind: RecordKind::Struct,
                members: Some(members),
            };
            r#type
                .borrow_mut()
                .attributes
                .push(Rc::new(RefCell::new(Attribute {
                    file_id,
                    span,
                    prefix_name: None,
                    name: "type_before_legalize".to_string(),
                    kind: AttributeKind::TypeBeforeLegalize { origin_type },
                })));
        }

        Ok(())
    }
}
