use crate::{
    ast::{Initializer, InitializerKind},
    codegen::riscv::{
        CodeGen, FP_REG,
        instruction::{Opcode, Operand},
    },
    ctype::{
        RecordKind, Type, TypeKind, array_element, get_inner_type,
        layout::{ConstDesignation, compute_layout},
    },
    symtab::Symbol,
};
use codespan_reporting::diagnostic::Diagnostic;
use std::{cell::RefCell, rc::Rc};

impl CodeGen {
    pub fn compute_offset_and_symbol(
        &self,
        r#type: &Rc<RefCell<Type>>,
        designation: Vec<ConstDesignation>,
        base: usize,
    ) -> Result<(usize, Option<Rc<RefCell<Symbol>>>), Diagnostic<usize>> {
        let r#type = get_inner_type(r#type.clone());

        match designation.get(0) {
            Some(ConstDesignation::Subscript(index)) => {
                let element_type = array_element(r#type).unwrap();
                let element_size = element_type.borrow().size().unwrap();
                self.compute_offset_and_symbol(
                    &element_type,
                    designation[1..].to_vec(),
                    base + index * element_size,
                )
            }
            Some(ConstDesignation::MemberAccess(name)) => match &r#type.borrow().kind {
                TypeKind::Record {
                    kind: RecordKind::Struct,
                    members: Some(members),
                    ..
                } => {
                    let member = members.get(name).unwrap().clone();
                    let layout = compute_layout(r#type.clone()).unwrap();
                    let mut offset = 0;

                    for child in &layout.children {
                        match &child.designation {
                            Some(ConstDesignation::MemberAccess(a)) if a == name => {
                                offset = child.offset;
                                break;
                            }
                            //位域
                            None => {
                                if child.children.iter().any(|x| match &x.designation {
                                    Some(ConstDesignation::MemberAccess(a)) => a == name,
                                    _ => false,
                                }) {
                                    offset = child.offset;
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }

                    let (offset, symbol) = self.compute_offset_and_symbol(
                        &member.borrow().r#type,
                        designation[1..].to_vec(),
                        base + offset,
                    )?;
                    return if designation.len() == 1 {
                        Ok((offset, Some(member.clone())))
                    } else {
                        Ok((offset, symbol))
                    };
                }
                TypeKind::Record {
                    kind: RecordKind::Union,
                    members: Some(members),
                    ..
                } => {
                    for (_, member) in members {
                        if member.borrow().name == *name {
                            let (offset, symbol) = self.compute_offset_and_symbol(
                                &member.borrow().r#type,
                                designation[1..].to_vec(),
                                base,
                            )?;
                            return if designation.len() == 1 {
                                Ok((offset, Some(member.clone())))
                            } else {
                                Ok((offset, symbol))
                            };
                        }
                    }
                    unreachable!()
                }
                _ => unreachable!(),
            },
            None => Ok((base, None)),
        }
    }

    pub fn visit_initializer(
        &mut self,
        node: &Rc<RefCell<Initializer>>,
        base: Option<Operand>,
        symbol: &Option<Rc<RefCell<Symbol>>>,
    ) -> Result<Operand, Diagnostic<usize>> {
        let Initializer { kind, r#type, .. } = &*node.borrow();

        let base = if let Some(base) = base {
            base
        } else {
            let (_, function) = self.functions.get_index_mut(self.cur_function).unwrap();
            function.adjust_local_frame_size(
                r#type.borrow().size().unwrap(),
                r#type.borrow().align().unwrap(),
            );
            let base = Operand::Address {
                base: Box::new(FP_REG),
                offset: -(function.local_frame_size as i64),
            };

            base
        };

        match kind {
            InitializerKind::Braced(initializers) => {
                for (i, initializer) in initializers.iter().enumerate() {
                    if i > 0
                        && (node.borrow().r#type.borrow().is_union()
                            || node.borrow().r#type.borrow().is_scale())
                    {
                        break;
                    }
                    let designation =
                        ConstDesignation::from_designation(&initializer.borrow().designations)?;
                    let (offset, symbol) =
                        self.compute_offset_and_symbol(&r#type, designation, 0)?;
                    let ptr = self.assign_ireg()?;
                    self.add_instruction(
                        Opcode::Add,
                        &[ptr.clone(), base.clone(), Operand::Immediate(offset as i64)],
                    )?;
                    self.visit_initializer(initializer, Some(ptr), &symbol)?;
                }
            }
            InitializerKind::Expr(expr) => {
                let value = self.visit_expr(expr)?;
                self.store(&base, &value, r#type, &symbol)?;
            }
        }

        Ok(base)
    }
}
