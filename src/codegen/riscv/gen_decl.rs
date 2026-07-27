use crate::{
    ast::decl::{Declaration, DeclarationKind, FunctionSpecKind, StorageClassKind},
    codegen::riscv::{
        A0_REG, A1_REG, CodeGen, FA0_REG, FP_REG, FP_REG_INDEX, RA_REG_INDEX, SP_REG, SP_REG_INDEX,
        function::Function,
        instruction::{Opcode, Operand},
        reg_allocator::RegAllocator,
    },
    ctype::{TypeKind, get_inner_type},
    optimizer::constfolder::ConstFolder,
    symtab::Namespace,
};
use codespan_reporting::diagnostic::Diagnostic;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

impl CodeGen {
    pub fn visit_declaration(
        &mut self,
        node: &Rc<RefCell<Declaration>>,
    ) -> Result<(), Diagnostic<usize>> {
        let Declaration {
            name,
            storage_classes,
            kind,
            r#type,
            ..
        } = &*node.borrow();

        if name.len() == 0 {
            return Ok(());
        }

        match kind {
            DeclarationKind::Function {
                parameter_decls,
                function_specs,
                body,
                symtab,
            } => {
                if let Some(t) = symtab {
                    self.enter_scope(t.clone());
                }

                let symbol = self.lookup(Namespace::Ordinary, name).unwrap();
                let key = symbol.as_ptr() as usize;
                let index = if let Some((i, _, _)) = self.functions.get_full(&key) {
                    i
                } else {
                    self.functions
                        .insert_before(self.functions.len(), key, Function::new(name));
                    self.functions.len() - 1
                };

                self.symbol_values
                    .insert(key, Operand::Symbol(name.clone()));

                if let Some(body) = body {
                    if !(function_specs
                        .iter()
                        .any(|x| x.kind == FunctionSpecKind::Inline)
                        || storage_classes
                            .iter()
                            .any(|x| x.kind == StorageClassKind::Static))
                    {
                        //函数名是唯一的不会重名, 所以不需要重新调整Function.name
                        self.add_global(name, (None, r#type.clone()))?;
                    }

                    self.cur_function = index;

                    let prologue_block = self.append_basic_block("prologue")?;
                    self.position_at_end(&prologue_block)?;
                    let entry_block = self.append_basic_block("entry")?;
                    self.position_at_end(&entry_block)?;

                    //TODO 保存和恢复其它寄存器
                    let (_, function) = self.functions.get_index_mut(self.cur_function).unwrap();
                    //fp寄存器必须放在最后
                    for ireg_index in [RA_REG_INDEX, SP_REG_INDEX, FP_REG_INDEX] {
                        function.adjust_local_frame_size(self.xlen / 8, self.xlen / 8);
                        function
                            .ireg_saved
                            .insert(ireg_index, -(function.local_frame_size as i64));
                    }

                    for decl in parameter_decls.iter() {
                        self.visit_declaration(decl)?;
                    }

                    self.visit_stmt(body)?;

                    let epilogue_block = self.current_basic_block();

                    //先分配, 再写函数序言
                    //因为寄存器分配可能也需要使用栈空间
                    let (_, function) = self.functions.get_index_mut(self.cur_function).unwrap();
                    RegAllocator::new(function, self.xlen).allocate();

                    //函数序言
                    self.position_at_end(&prologue_block)?;

                    self.save_callee_regs()?;

                    let (_, function) = self.functions.get_index(self.cur_function).unwrap();
                    //16字节对齐
                    let frame_size =
                        (function.local_frame_size + function.arg_frame_size + 15) / 16 * 16;

                    self.add_instruction(Opcode::Move, &[FP_REG, SP_REG])?;
                    self.add_instruction(
                        Opcode::Add,
                        &[SP_REG, SP_REG, Operand::Immediate(-(frame_size as i64))],
                    )?;

                    //函数尾声
                    //默认返回值
                    self.position_at_end(&epilogue_block)?;

                    let return_type = match &get_inner_type(r#type.clone()).borrow().kind {
                        TypeKind::Function { return_type, .. } => return_type.clone(),
                        _ => unreachable!(),
                    };

                    let xsize = self.xlen / 8;

                    match &get_inner_type(return_type).borrow().kind {
                        t if t.is_float_type() => {
                            self.add_instruction(
                                Opcode::LoadImm,
                                &[A0_REG, Operand::Immediate(0)],
                            )?;
                            self.add_instruction(
                                match self.xlen {
                                    32 => Opcode::FCvtSLU,
                                    64 => Opcode::FCvtSWU,
                                    _ => unreachable!(),
                                },
                                &[FA0_REG, A0_REG],
                            )?;
                        }
                        t if t.is_double() => {
                            self.add_instruction(
                                Opcode::LoadImm,
                                &[A0_REG, Operand::Immediate(0)],
                            )?;
                            self.add_instruction(
                                match self.xlen {
                                    32 => Opcode::FCvtDLU,
                                    64 => Opcode::FCvtDWU,
                                    _ => unreachable!(),
                                },
                                &[FA0_REG, A0_REG],
                            )?;
                        }
                        t if t.is_scale() => {
                            self.add_instruction(
                                Opcode::LoadImm,
                                &[A0_REG, Operand::Immediate(0)],
                            )?;
                        }
                        t if t.is_aggregate() => {
                            let size = t.size().unwrap();
                            if size > xsize * 2 {
                                self.call_memset(
                                    &A0_REG,
                                    &Operand::Immediate(0),
                                    &Operand::Immediate(size as i64),
                                )?;
                            } else {
                                self.add_instruction(
                                    Opcode::LoadImm,
                                    &[A0_REG, Operand::Immediate(0)],
                                )?;
                                if size > xsize {
                                    self.add_instruction(
                                        Opcode::LoadImm,
                                        &[A1_REG, Operand::Immediate(0)],
                                    )?;
                                }
                            }
                        }
                        _ => {}
                    }

                    self.restore_callee_regs()?;

                    self.add_instruction(Opcode::Ret, &[])?;

                    self.cur_function = usize::MAX;
                }

                if let Some(_) = symtab {
                    self.leave_scope();
                }
            }
            DeclarationKind::Var { initializer } => {
                let symbol = self.lookup(Namespace::Ordinary, name).unwrap();
                let key = symbol.as_ptr() as usize;

                let value = match (
                    self.functions.get_index_mut(self.cur_function),
                    storage_classes
                        .iter()
                        .any(|x| x.kind == StorageClassKind::Static),
                ) {
                    (Some((_, function)), false) => {
                        let symbol_type = symbol.borrow().r#type.clone();
                        function.adjust_local_frame_size(
                            symbol_type.borrow().size().unwrap(),
                            symbol_type.borrow().align().unwrap(),
                        );
                        let value = Operand::Address {
                            base: Box::new(FP_REG),
                            offset: -(function.local_frame_size as i64),
                        };

                        if let Some(initializer) = initializer {
                            let init_value = self.visit_initializer(initializer, None, &None)?;
                            let t = self.load(&init_value, r#type, &Some(symbol.clone()))?;
                            self.store(&value, &t, r#type, &Some(symbol))?;
                        }

                        value
                    }
                    _ => {
                        let value = if let Some(initializer) = initializer {
                            ConstFolder::new()
                                .visit_initializer(initializer.clone(), HashMap::new())?;
                            Some(initializer.borrow().value.clone())
                        } else {
                            None
                        };

                        let name = self.add_global(&name, (value, r#type.clone()))?;
                        let value = Operand::Symbol(name.to_string());

                        value
                    }
                };

                self.symbol_values.insert(key, value);
            }
            DeclarationKind::Parameter => {
                let xsize = self.xlen / 8;

                let symbol = self.lookup(Namespace::Ordinary, name).unwrap();
                let key = symbol.as_ptr() as usize;

                let (_, function) = self.functions.get_index_mut(self.cur_function).unwrap();
                let symbol_type = symbol.borrow().r#type.clone();
                function.adjust_local_frame_size(
                    symbol_type.borrow().size().unwrap(),
                    symbol_type.borrow().align().unwrap(),
                );
                let value = Operand::Address {
                    base: Box::new(FP_REG),
                    offset: -(function.local_frame_size as i64),
                };
                let frame_size = function.local_frame_size;

                let mut ireg_used = function.ireg_used;
                let mut freg_used = function.freg_used;

                let store_opcode = match self.xlen {
                    32 => Opcode::StoreW,
                    64 => Opcode::StoreD,
                    _ => unreachable!(),
                };

                let arg_type = get_inner_type(r#type.clone());
                match arg_type.borrow().kind.clone() {
                    t if t.is_float_type() => {
                        if freg_used < 8 {
                            self.add_instruction(
                                Opcode::FStoreS,
                                &[Operand::FPReg(10 + freg_used), value.clone()],
                            )?;
                            freg_used += 1;
                        } else {
                            let t = self.pop_arg(&arg_type)?;
                            let t2 = self.assign_freg()?;
                            self.add_instruction(
                                match self.xlen {
                                    32 => Opcode::FMoveSW,
                                    64 => Opcode::FMoveSL,
                                    _ => unreachable!(),
                                },
                                &[t2.clone(), t],
                            )?;
                            self.add_instruction(Opcode::FStoreS, &[t2, value.clone()])?;
                        }
                    }
                    t if t.is_double() => {
                        if freg_used < 8 {
                            self.add_instruction(
                                Opcode::FStoreD,
                                &[Operand::FPReg(10 + freg_used), value.clone()],
                            )?;
                            freg_used += 1;
                        } else {
                            let t = self.pop_arg(&arg_type)?;
                            let t2 = self.assign_freg()?;
                            self.add_instruction(
                                match self.xlen {
                                    32 => Opcode::FMoveDW,
                                    64 => Opcode::FMoveDL,
                                    _ => unreachable!(),
                                },
                                &[t2.clone(), t],
                            )?;
                            self.add_instruction(Opcode::FStoreD, &[t2, value.clone()])?;
                        }
                    }
                    t if t.is_scale() => {
                        if ireg_used < 8 {
                            self.add_instruction(
                                store_opcode,
                                &[Operand::IntReg(10 + ireg_used), value.clone()],
                            )?;
                            ireg_used += 1;
                        } else {
                            let t = self.pop_arg(&arg_type)?;
                            self.add_instruction(store_opcode, &[t, value.clone()])?;
                        }
                    }
                    t if t.is_aggregate() => {
                        let size = t.size().unwrap();

                        if size > xsize * 2 {
                            let ptr = self.pop_arg(&arg_type)?;
                            self.call_memcpy(&value, &ptr, &Operand::Immediate(size as i64))?;
                        } else {
                            if ireg_used < 8 {
                                self.add_instruction(
                                    store_opcode,
                                    &[Operand::IntReg(10 + ireg_used), value.clone()],
                                )?;
                                ireg_used += 1;
                            } else {
                                let t = self.pop_arg(&arg_type)?;
                                self.add_instruction(store_opcode, &[t, value.clone()])?;
                            }

                            if size > xsize {
                                let value = Operand::Address {
                                    base: Box::new(FP_REG),
                                    offset: xsize as i64 - (frame_size as i64),
                                };
                                if ireg_used < 8 {
                                    self.add_instruction(
                                        store_opcode,
                                        &[Operand::IntReg(10 + ireg_used), value.clone()],
                                    )?;
                                    ireg_used += 1;
                                } else {
                                    let t = self.pop_arg(&arg_type)?;
                                    self.add_instruction(store_opcode, &[t, value.clone()])?;
                                }
                            }
                        }
                    }
                    _ => {}
                }

                self.symbol_values.insert(key, value);

                let (_, function) = self.functions.get_index_mut(self.cur_function).unwrap();
                function.ireg_used = ireg_used;
                function.freg_used = freg_used;
            }
            _ => {}
        }

        Ok(())
    }
}
