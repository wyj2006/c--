use crate::{
    ast::{
        decl::StorageClassKind,
        expr::{BinOpKind, CastMethod, Expr, ExprKind, UnaryOpKind},
    },
    codegen::riscv::{
        A0_REG, A1_REG, CodeGen, FA0_REG, FP_REG,
        instruction::{Opcode, Operand},
    },
    ctype::{RecordKind, Type, TypeKind, get_inner_type, layout::ConstDesignation, pointee},
    optimizer::constfolder::ConstFolder,
    symtab::{Namespace, SymbolKind},
};
use codespan_reporting::diagnostic::Diagnostic;
use num::ToPrimitive;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

impl CodeGen {
    pub fn visit_expr(&mut self, node: &Rc<RefCell<Expr>>) -> Result<Operand, Diagnostic<usize>> {
        if !matches!(node.borrow().kind, ExprKind::String { .. }) && !node.borrow().has_side_effects
        {
            //TODO 优化ConstFolder的调用
            ConstFolder::new().visit_expr(node.clone(), HashMap::new())?;
            match self.variant_to_operand(&node.borrow().value, &node.borrow().r#type) {
                Ok(t) => return Ok(t),
                Err(_) => {}
            }
        }

        let Expr {
            file_id,
            span,
            kind,
            r#type,
            symbol,
            value,
            ..
        } = &*node.borrow();

        match kind {
            ExprKind::Name(name) => {
                let symbol = match self.lookup(Namespace::Ordinary, name) {
                    Some(t) => t,
                    None => panic!("{name} not defined"), //一般是内建函数
                };
                match &symbol.borrow().kind {
                    SymbolKind::EnumConst { value } => {
                        Ok(Operand::Immediate(value.to_i64().unwrap_or(i64::MAX)))
                    }
                    _ => {
                        let key = symbol.as_ptr() as usize;
                        match self.symbol_values.get(&key) {
                            Some(t) => Ok(t.clone()),
                            None => panic!("{name} not in symbol_values"),
                        }
                    }
                }
            }
            ExprKind::String { .. } => {
                let name = self.add_global("string", (Some(value.clone()), r#type.clone()))?;
                Ok(Operand::Symbol(name))
            }
            ExprKind::GenericSelection { assocs, .. } => {
                for assoc in assocs {
                    if assoc.borrow().is_selected {
                        return self.visit_expr(&assoc.borrow().expr);
                    }
                }
                unreachable!()
            }
            ExprKind::Conditional {
                condition,
                true_expr,
                false_expr,
            } => {
                let cond_block = self.current_basic_block();
                let true_block = self.append_basic_block("cond_true")?;
                self.position_at_end(&true_block)?;
                let false_block = self.append_basic_block("cond_false")?;
                self.position_at_end(&false_block)?;
                let merge_block = self.append_basic_block("cond_merge")?;

                let value = match &r#type.borrow().kind {
                    t if t.is_integer() || t.is_pointer() => self.assign_ireg()?,
                    t if t.is_real_float() => self.assign_freg()?,
                    _ => unreachable!(),
                };

                self.position_at_end(&cond_block)?;
                let cond = self.visit_expr(condition)?;
                self.add_instruction(
                    Opcode::BEqZ,
                    &[cond.clone(), Operand::Symbol(false_block.clone())],
                )?;
                self.add_instruction(Opcode::Jump, &[Operand::Symbol(true_block.clone())])?;

                self.position_at_end(&true_block)?;
                let true_value = self.visit_expr(true_expr)?;
                self.add_instruction(Opcode::Move, &[value.clone(), true_value])?;
                self.add_instruction(Opcode::Jump, &[Operand::Symbol(merge_block.clone())])?;

                self.position_at_end(&false_block)?;
                let false_value = self.visit_expr(false_expr)?;
                self.add_instruction(Opcode::Move, &[value.clone(), false_value])?;
                self.add_instruction(Opcode::Jump, &[Operand::Symbol(merge_block.clone())])?;

                self.position_at_end(&merge_block)?;

                Ok(value)
            }
            ExprKind::Subscript { target, index } => {
                let target_type_kind = target.borrow().r#type.borrow().kind.clone();
                let index_type_kind = index.borrow().r#type.borrow().kind.clone();

                let (base, index) = match (target_type_kind, index_type_kind) {
                    (a, b) if a.is_pointer() && b.is_integer() => {
                        (self.visit_expr(target)?, self.visit_expr(index)?)
                    }
                    (a, b) if a.is_integer() && b.is_pointer() => {
                        (self.visit_expr(index)?, self.visit_expr(target)?)
                    }
                    _ => unreachable!(),
                };
                let element_size = r#type.borrow().size().unwrap();

                let offset = self.assign_ireg()?;
                self.add_instruction(
                    Opcode::Mul,
                    &[
                        offset.clone(),
                        index.clone(),
                        Operand::Immediate(element_size as i64),
                    ],
                )?;

                let addr = self.assign_ireg()?;
                self.add_instruction(Opcode::Add, &[addr.clone(), base.clone(), offset.clone()])?;

                Ok(addr)
            }
            ExprKind::MemberAccess {
                target,
                is_arrow,
                name,
            } => {
                let base = self.visit_expr(target)?;

                let inner_type = if *is_arrow {
                    get_inner_type(pointee(target.borrow().r#type.clone()).unwrap())
                } else {
                    get_inner_type(target.borrow().r#type.clone())
                };
                let Type {
                    kind: TypeKind::Record { kind, .. },
                    ..
                } = &*inner_type.borrow()
                else {
                    unreachable!()
                };

                if let RecordKind::Union = kind {
                    Ok(base)
                } else {
                    let (offset, _) = self.compute_offset_and_symbol(
                        &inner_type,
                        vec![ConstDesignation::MemberAccess(name.clone())],
                        0,
                    )?;

                    let addr = self.assign_ireg()?;
                    self.add_instruction(
                        Opcode::Add,
                        &[addr.clone(), base, Operand::Immediate(offset as i64)],
                    )?;
                    Ok(addr)
                }
            }
            ExprKind::UnaryOp { op, operand } => match op {
                UnaryOpKind::AddressOf => {
                    let operand_kind = operand.borrow().kind.clone();
                    match &operand_kind {
                        ExprKind::UnaryOp {
                            op: UnaryOpKind::Dereference,
                            operand,
                        } => Ok(self.visit_expr(operand)?),
                        _ => match self.visit_expr(operand)? {
                            t @ Operand::Symbol(_) => {
                                let value = self.assign_ireg()?;
                                self.add_instruction(
                                    Opcode::LoadAddr,
                                    &[value.clone(), t.clone()],
                                )?;
                                Ok(value)
                            }
                            Operand::Address { base, offset } => {
                                let value = self.assign_ireg()?;
                                self.add_instruction(
                                    Opcode::Add,
                                    &[value.clone(), (*base).clone(), Operand::Immediate(offset)],
                                )?;
                                Ok(value)
                            }
                            //操作数是左值的话, 它的求值结果本身就是地址
                            t => Ok(t),
                        },
                    }
                }
                UnaryOpKind::Dereference => {
                    let operand_value = self.visit_expr(operand)?;
                    //解引用的结果是左值, 而operand_value就是它的地址
                    Ok(operand_value)
                }
                UnaryOpKind::BitNot => {
                    let operand_value = self.visit_expr(operand)?;
                    let value = self.assign_ireg()?;
                    self.add_instruction(Opcode::BitNot, &[value.clone(), operand_value])?;
                    Ok(value)
                }
                UnaryOpKind::Negative => {
                    let operand_value = self.visit_expr(operand)?;
                    match &r#type.borrow().kind {
                        t if t.is_integer() => {
                            let value = self.assign_ireg()?;
                            self.add_instruction(Opcode::Neg, &[value.clone(), operand_value])?;
                            Ok(value)
                        }
                        t if t.is_float_type() => {
                            let value = self.assign_freg()?;
                            self.add_instruction(Opcode::FNegS, &[value.clone(), operand_value])?;
                            Ok(value)
                        }
                        t if t.is_double() => {
                            let value = self.assign_freg()?;
                            self.add_instruction(Opcode::FNegD, &[value.clone(), operand_value])?;
                            Ok(value)
                        }
                        _ => unreachable!(),
                    }
                }
                UnaryOpKind::Not => {
                    let operand_value = self.visit_expr(operand)?;
                    let value = self.assign_ireg()?;
                    self.add_instruction(Opcode::SetEqZ, &[value.clone(), operand_value])?;
                    Ok(value)
                }
                UnaryOpKind::Positive => Ok(self.visit_expr(operand))?,
                UnaryOpKind::PostfixDec => {
                    let operand_ptr = self.visit_expr(operand)?;
                    let operand_value = self.load(&operand_ptr, r#type, symbol)?;

                    let offset = match pointee(operand.borrow().r#type.clone()) {
                        Some(t) => Operand::Immediate(-(t.borrow().size().unwrap() as i64)),
                        None => Operand::Immediate(-1),
                    };

                    let value = self.assign_ireg()?;
                    self.add_instruction(
                        Opcode::Add,
                        &[value.clone(), operand_value.clone(), offset],
                    )?;
                    self.store(&operand_ptr, &value, r#type, symbol)?;

                    Ok(operand_value)
                }
                UnaryOpKind::PostfixInc => {
                    let operand_ptr = self.visit_expr(operand)?;
                    let operand_value = self.load(&operand_ptr, r#type, symbol)?;

                    let offset = match pointee(operand.borrow().r#type.clone()) {
                        Some(t) => Operand::Immediate(t.borrow().size().unwrap() as i64),
                        None => Operand::Immediate(1),
                    };

                    let value = self.assign_ireg()?;
                    self.add_instruction(
                        Opcode::Add,
                        &[value.clone(), operand_value.clone(), offset],
                    )?;
                    self.store(&operand_ptr, &value, r#type, symbol)?;

                    Ok(operand_value)
                }
                UnaryOpKind::PrefixDec => {
                    let operand_ptr = self.visit_expr(operand)?;
                    let operand_value = self.load(&operand_ptr, r#type, symbol)?;

                    let offset = match pointee(operand.borrow().r#type.clone()) {
                        Some(t) => Operand::Immediate(-(t.borrow().size().unwrap() as i64)),
                        None => Operand::Immediate(-1),
                    };

                    let value = self.assign_ireg()?;
                    self.add_instruction(
                        Opcode::Add,
                        &[value.clone(), operand_value.clone(), offset],
                    )?;
                    self.store(&operand_ptr, &value, r#type, symbol)?;

                    Ok(value)
                }
                UnaryOpKind::PrefixInc => {
                    let operand_ptr = self.visit_expr(operand)?;
                    let operand_value = self.load(&operand_ptr, r#type, symbol)?;

                    let offset = match pointee(operand.borrow().r#type.clone()) {
                        Some(t) => Operand::Immediate(t.borrow().size().unwrap() as i64),
                        None => Operand::Immediate(1),
                    };

                    let value = self.assign_ireg()?;
                    self.add_instruction(
                        Opcode::Add,
                        &[value.clone(), operand_value.clone(), offset],
                    )?;
                    self.store(&operand_ptr, &value, r#type, symbol)?;

                    Ok(value)
                }
            },
            ExprKind::BinOp { op, left, right } => {
                match op {
                    BinOpKind::Add => {
                        let left_value = self.visit_expr(left)?;
                        let right_value = self.visit_expr(right)?;
                        match &r#type.borrow().kind {
                            t if t.is_integer() || t.is_pointer() => {
                                let value = self.assign_ireg()?;
                                self.add_instruction(
                                    Opcode::Add,
                                    &[value.clone(), left_value, right_value],
                                )?;
                                Ok(value)
                            }
                            t if t.is_float_type() => {
                                let value = self.assign_freg()?;
                                self.add_instruction(
                                    Opcode::FAddS,
                                    &[value.clone(), left_value, right_value],
                                )?;
                                Ok(value)
                            }
                            t if t.is_double() => {
                                let value = self.assign_freg()?;
                                self.add_instruction(
                                    Opcode::FAddD,
                                    &[value.clone(), left_value, right_value],
                                )?;
                                Ok(value)
                            }
                            _ => unreachable!(),
                        }
                    }
                    BinOpKind::Sub => {
                        let left_value = self.visit_expr(left)?;
                        let right_value = self.visit_expr(right)?;
                        match &r#type.borrow().kind {
                            t if t.is_integer() || t.is_pointer() => {
                                let value = self.assign_ireg()?;
                                self.add_instruction(
                                    Opcode::Sub,
                                    &[value.clone(), left_value, right_value],
                                )?;
                                Ok(value)
                            }
                            t if t.is_float_type() => {
                                let value = self.assign_freg()?;
                                self.add_instruction(
                                    Opcode::FSubS,
                                    &[value.clone(), left_value, right_value],
                                )?;
                                Ok(value)
                            }
                            t if t.is_double() => {
                                let value = self.assign_freg()?;
                                self.add_instruction(
                                    Opcode::FSubD,
                                    &[value.clone(), left_value, right_value],
                                )?;
                                Ok(value)
                            }
                            _ => unreachable!(),
                        }
                    }
                    BinOpKind::Mul => {
                        let left_value = self.visit_expr(left)?;
                        let right_value = self.visit_expr(right)?;
                        match &r#type.borrow().kind {
                            t if t.is_integer() => {
                                let value = self.assign_ireg()?;
                                self.add_instruction(
                                    Opcode::Mul,
                                    &[value.clone(), left_value, right_value],
                                )?;
                                Ok(value)
                            }
                            t if t.is_float_type() => {
                                let value = self.assign_freg()?;
                                self.add_instruction(
                                    Opcode::FMulS,
                                    &[value.clone(), left_value, right_value],
                                )?;
                                Ok(value)
                            }
                            t if t.is_double() => {
                                let value = self.assign_freg()?;
                                self.add_instruction(
                                    Opcode::FMulD,
                                    &[value.clone(), left_value, right_value],
                                )?;
                                Ok(value)
                            }
                            _ => unreachable!(),
                        }
                    }
                    BinOpKind::Div => {
                        let left_value = self.visit_expr(left)?;
                        let right_value = self.visit_expr(right)?;
                        match &r#type.borrow().kind {
                            t if t.is_integer() => {
                                let value = self.assign_ireg()?;
                                self.add_instruction(
                                    if t.is_unsigned().unwrap() {
                                        Opcode::DivU
                                    } else {
                                        Opcode::Div
                                    },
                                    &[value.clone(), left_value, right_value],
                                )?;
                                Ok(value)
                            }
                            t if t.is_float_type() => {
                                let value = self.assign_freg()?;
                                self.add_instruction(
                                    Opcode::FDivS,
                                    &[value.clone(), left_value, right_value],
                                )?;
                                Ok(value)
                            }
                            t if t.is_double() => {
                                let value = self.assign_freg()?;
                                self.add_instruction(
                                    Opcode::FDivD,
                                    &[value.clone(), left_value, right_value],
                                )?;
                                Ok(value)
                            }
                            _ => unreachable!(),
                        }
                    }
                    BinOpKind::Mod => {
                        let left_value = self.visit_expr(left)?;
                        let right_value = self.visit_expr(right)?;

                        let value = self.assign_ireg()?;
                        self.add_instruction(
                            if r#type.borrow().is_unsigned().unwrap() {
                                Opcode::RemU
                            } else {
                                Opcode::Rem
                            },
                            &[value.clone(), left_value, right_value],
                        )?;
                        Ok(value)
                    }
                    BinOpKind::LShift => {
                        let left_value = self.visit_expr(left)?;
                        let right_value = self.visit_expr(right)?;

                        let value = self.assign_ireg()?;
                        self.add_instruction(
                            Opcode::LShift,
                            &[value.clone(), left_value, right_value],
                        )?;
                        Ok(value)
                    }
                    BinOpKind::RShift => {
                        let left_value = self.visit_expr(left)?;
                        let right_value = self.visit_expr(right)?;

                        let value = self.assign_ireg()?;
                        self.add_instruction(
                            if left.borrow().r#type.borrow().is_unsigned().unwrap() {
                                Opcode::RShiftL
                            } else {
                                Opcode::RShiftA
                            },
                            &[value.clone(), left_value, right_value],
                        )?;
                        Ok(value)
                    }
                    BinOpKind::Lt => {
                        let left_value = self.visit_expr(left)?;
                        let right_value = self.visit_expr(right)?;
                        match &left.borrow().r#type.borrow().kind {
                            t if t.is_integer() => {
                                let value = self.assign_ireg()?;
                                self.add_instruction(
                                    if t.is_unsigned().unwrap() {
                                        Opcode::SetLtU
                                    } else {
                                        Opcode::SetLt
                                    },
                                    &[value.clone(), left_value, right_value],
                                )?;
                                Ok(value)
                            }
                            t if t.is_float_type() => {
                                let value = self.assign_ireg()?;
                                self.add_instruction(
                                    Opcode::FLtS,
                                    &[value.clone(), left_value, right_value],
                                )?;
                                Ok(value)
                            }
                            t if t.is_double() => {
                                let value = self.assign_ireg()?;
                                self.add_instruction(
                                    Opcode::FLtD,
                                    &[value.clone(), left_value, right_value],
                                )?;
                                Ok(value)
                            }
                            _ => unreachable!(),
                        }
                    }
                    BinOpKind::Le => {
                        let left_value = self.visit_expr(left)?;
                        let right_value = self.visit_expr(right)?;

                        let value = self.assign_ireg()?;
                        match &left.borrow().r#type.borrow().kind {
                            t if t.is_integer() => {
                                // a<=b => not(a-b>0)
                                let t = self.assign_ireg()?;
                                self.add_instruction(
                                    Opcode::Sub,
                                    &[t.clone(), left_value.clone(), right_value.clone()],
                                )?;
                                self.add_instruction(Opcode::SetGtZ, &[value.clone(), t])?;
                                self.add_instruction(
                                    Opcode::SetEqZ,
                                    &[value.clone(), value.clone()],
                                )?;
                            }
                            t if t.is_float_type() => {
                                self.add_instruction(
                                    Opcode::FLeS,
                                    &[value.clone(), left_value, right_value],
                                )?;
                            }
                            t if t.is_double() => {
                                self.add_instruction(
                                    Opcode::FLeD,
                                    &[value.clone(), left_value, right_value],
                                )?;
                            }
                            _ => unreachable!(),
                        }
                        Ok(value)
                    }
                    BinOpKind::Gt => {
                        let left_value = self.visit_expr(left)?;
                        let right_value = self.visit_expr(right)?;

                        let value = self.assign_ireg()?;
                        match &left.borrow().r#type.borrow().kind {
                            t if t.is_integer() => {
                                let t = self.assign_ireg()?;
                                self.add_instruction(
                                    Opcode::Sub,
                                    &[t.clone(), left_value, right_value],
                                )?;
                                self.add_instruction(Opcode::SetGtZ, &[value.clone(), t])?;
                            }
                            t if t.is_float_type() => {
                                let t = self.assign_freg()?;
                                self.add_instruction(
                                    Opcode::FSubS,
                                    &[t.clone(), left_value, right_value],
                                )?;
                                self.add_instruction(Opcode::FLeS, &[value.clone(), t])?;
                                self.add_instruction(
                                    Opcode::SetEqZ,
                                    &[value.clone(), value.clone()],
                                )?;
                            }
                            t if t.is_double() => {
                                let t = self.assign_freg()?;
                                self.add_instruction(
                                    Opcode::FSubD,
                                    &[t.clone(), left_value, right_value],
                                )?;
                                self.add_instruction(Opcode::FLeD, &[value.clone(), t])?;
                                self.add_instruction(
                                    Opcode::SetEqZ,
                                    &[value.clone(), value.clone()],
                                )?;
                            }
                            _ => unreachable!(),
                        }
                        Ok(value)
                    }
                    BinOpKind::Ge => {
                        let left_value = self.visit_expr(left)?;
                        let right_value = self.visit_expr(right)?;

                        let value = self.assign_ireg()?;
                        match &left.borrow().r#type.borrow().kind {
                            t if t.is_integer() => {
                                let t = self.assign_ireg()?;
                                self.add_instruction(
                                    Opcode::Sub,
                                    &[t.clone(), left_value, right_value],
                                )?;
                                self.add_instruction(Opcode::SetLtZ, &[value.clone(), t])?;
                                self.add_instruction(
                                    Opcode::SetEqZ,
                                    &[value.clone(), value.clone()],
                                )?;
                            }
                            t if t.is_float_type() => {
                                let t = self.assign_freg()?;
                                self.add_instruction(
                                    Opcode::FSubS,
                                    &[t.clone(), left_value, right_value],
                                )?;
                                self.add_instruction(Opcode::FLtS, &[value.clone(), t])?;
                                self.add_instruction(
                                    Opcode::SetEqZ,
                                    &[value.clone(), value.clone()],
                                )?;
                            }
                            t if t.is_double() => {
                                let t = self.assign_freg()?;
                                self.add_instruction(
                                    Opcode::FSubD,
                                    &[t.clone(), left_value, right_value],
                                )?;
                                self.add_instruction(Opcode::FLtD, &[value.clone(), t])?;
                                self.add_instruction(
                                    Opcode::SetEqZ,
                                    &[value.clone(), value.clone()],
                                )?;
                            }
                            _ => unreachable!(),
                        }
                        Ok(value)
                    }
                    BinOpKind::Eq => {
                        let left_value = self.visit_expr(left)?;
                        let right_value = self.visit_expr(right)?;

                        let value = self.assign_ireg()?;
                        match &left.borrow().r#type.borrow().kind {
                            t if t.is_integer() => {
                                let t = self.assign_ireg()?;
                                self.add_instruction(
                                    Opcode::Sub,
                                    &[t.clone(), left_value, right_value],
                                )?;
                                self.add_instruction(Opcode::SetEqZ, &[value.clone(), t])?;
                            }
                            t if t.is_float_type() => {
                                self.add_instruction(
                                    Opcode::FEqS,
                                    &[value.clone(), left_value, right_value],
                                )?;
                            }
                            t if t.is_double() => {
                                self.add_instruction(
                                    Opcode::FEqD,
                                    &[value.clone(), left_value, right_value],
                                )?;
                            }
                            _ => unreachable!(),
                        }
                        Ok(value)
                    }
                    BinOpKind::Neq => {
                        let left_value = self.visit_expr(left)?;
                        let right_value = self.visit_expr(right)?;

                        let value = self.assign_ireg()?;
                        match &left.borrow().r#type.borrow().kind {
                            t if t.is_integer() => {
                                let t = self.assign_ireg()?;
                                self.add_instruction(
                                    Opcode::Sub,
                                    &[t.clone(), left_value, right_value],
                                )?;
                                self.add_instruction(Opcode::SetNeqZ, &[value.clone(), t])?;
                            }
                            t if t.is_float_type() => {
                                self.add_instruction(
                                    Opcode::FEqS,
                                    &[value.clone(), left_value, right_value],
                                )?;
                                self.add_instruction(
                                    Opcode::SetEqZ,
                                    &[value.clone(), value.clone()],
                                )?;
                            }
                            t if t.is_double() => {
                                self.add_instruction(
                                    Opcode::FEqD,
                                    &[value.clone(), left_value, right_value],
                                )?;
                                self.add_instruction(
                                    Opcode::SetEqZ,
                                    &[value.clone(), value.clone()],
                                )?;
                            }
                            _ => unreachable!(),
                        }
                        Ok(value)
                    }
                    BinOpKind::BitAnd => {
                        let left_value = self.visit_expr(left)?;
                        let right_value = self.visit_expr(right)?;

                        let value = self.assign_ireg()?;
                        self.add_instruction(
                            Opcode::And,
                            &[value.clone(), left_value, right_value],
                        )?;
                        Ok(value)
                    }
                    BinOpKind::BitXOr => {
                        let left_value = self.visit_expr(left)?;
                        let right_value = self.visit_expr(right)?;

                        let value = self.assign_ireg()?;
                        self.add_instruction(
                            Opcode::Xor,
                            &[value.clone(), left_value, right_value],
                        )?;
                        Ok(value)
                    }
                    BinOpKind::BitOr => {
                        let left_value = self.visit_expr(left)?;
                        let right_value = self.visit_expr(right)?;

                        let value = self.assign_ireg()?;
                        self.add_instruction(
                            Opcode::Or,
                            &[value.clone(), left_value, right_value],
                        )?;
                        Ok(value)
                    }
                    BinOpKind::And => {
                        let value = self.assign_ireg()?;

                        let left_block = self.current_basic_block();
                        let right_block = self.append_basic_block("and_right")?;
                        self.position_at_end(&right_block)?;
                        let merge_block = self.append_basic_block("and_merge")?;

                        self.position_at_end(&left_block)?;
                        let left_value = self.visit_expr(left)?;
                        self.add_instruction(Opcode::SetNeqZ, &[value.clone(), left_value])?;
                        self.add_instruction(
                            Opcode::BEqZ,
                            &[value.clone(), Operand::Symbol(merge_block.clone())],
                        )?;

                        self.position_at_end(&right_block)?;
                        let right_value = self.visit_expr(right)?;
                        self.add_instruction(Opcode::SetNeqZ, &[value.clone(), right_value])?;
                        self.add_instruction(
                            Opcode::Jump,
                            &[Operand::Symbol(merge_block.clone())],
                        )?;

                        self.position_at_end(&merge_block)?;

                        Ok(value)
                    }
                    BinOpKind::Or => {
                        let value = self.assign_ireg()?;

                        let left_block = self.current_basic_block();
                        let right_block = self.append_basic_block("or_right")?;
                        self.position_at_end(&right_block)?;
                        let merge_block = self.append_basic_block("or_merge")?;

                        self.position_at_end(&left_block)?;
                        let left_value = self.visit_expr(left)?;
                        self.add_instruction(Opcode::SetNeqZ, &[value.clone(), left_value])?;
                        self.add_instruction(
                            Opcode::BNeqZ,
                            &[value.clone(), Operand::Symbol(merge_block.clone())],
                        )?;

                        self.position_at_end(&right_block)?;
                        let right_value = self.visit_expr(right)?;
                        self.add_instruction(Opcode::SetNeqZ, &[value.clone(), right_value])?;
                        self.add_instruction(
                            Opcode::Jump,
                            &[Operand::Symbol(merge_block.clone())],
                        )?;

                        self.position_at_end(&merge_block)?;

                        Ok(value)
                    }
                    BinOpKind::Comma => {
                        self.visit_expr(left)?;
                        Ok(self.visit_expr(right)?)
                    }
                    BinOpKind::Assign => {
                        let ptr = self.visit_expr(left)?;
                        let value = self.visit_expr(right)?;
                        self.store(&ptr, &value, r#type, symbol)?;
                        Ok(value)
                    }
                    BinOpKind::MulAssign
                    | BinOpKind::DivAssign
                    | BinOpKind::ModAssign
                    | BinOpKind::AddAssign
                    | BinOpKind::SubAssign
                    | BinOpKind::LShiftAssign
                    | BinOpKind::RShiftAssign
                    | BinOpKind::BitAndAssign
                    | BinOpKind::BitOrAssign
                    | BinOpKind::BitXOrAssign => {
                        let eq_expr = Rc::new(RefCell::new(Expr {
                            r#type: r#type.clone(),
                            ..Expr::new(
                                *file_id,
                                *span,
                                ExprKind::BinOp {
                                    op: match op {
                                        BinOpKind::MulAssign => BinOpKind::Mul,
                                        BinOpKind::DivAssign => BinOpKind::Div,
                                        BinOpKind::ModAssign => BinOpKind::Mod,
                                        BinOpKind::AddAssign => BinOpKind::Add,
                                        BinOpKind::SubAssign => BinOpKind::Sub,
                                        BinOpKind::LShiftAssign => BinOpKind::LShift,
                                        BinOpKind::RShiftAssign => BinOpKind::RShift,
                                        BinOpKind::BitAndAssign => BinOpKind::BitAnd,
                                        BinOpKind::BitOrAssign => BinOpKind::BitOr,
                                        BinOpKind::BitXOrAssign => BinOpKind::BitXOr,
                                        _ => unreachable!(),
                                    },
                                    left: Rc::new(RefCell::new(Expr {
                                        r#type: Rc::clone(&left.borrow().r#type),
                                        ..Expr::new(
                                            left.borrow().file_id,
                                            left.borrow().span,
                                            ExprKind::Cast {
                                                is_implicit: true,
                                                target: Rc::clone(left),
                                                decls: vec![],
                                                method: CastMethod::LToRValue,
                                            },
                                        )
                                    })),
                                    right: Rc::clone(right),
                                },
                            )
                        }));
                        let ptr = self.visit_expr(left)?;
                        let value = self.visit_expr(&eq_expr)?;
                        self.store(&ptr, &value, r#type, symbol)?;
                        Ok(value)
                    }
                }
            }
            ExprKind::Cast { target, method, .. } => {
                let target_value = self.visit_expr(target)?;
                match method {
                    CastMethod::Nothing
                    | CastMethod::ArrayToPtr
                    | CastMethod::PtrToPtr
                    | CastMethod::PtrToInt
                    | CastMethod::IntToPtr
                    | CastMethod::FuncToPtr => Ok(target_value),
                    CastMethod::LToRValue => {
                        Ok(self.load(&target_value, &target.borrow().r#type, symbol)?)
                    }
                    CastMethod::ToBool => {
                        Ok(self.to_bool(&target_value, Some(&target.borrow().r#type))?)
                    }
                    CastMethod::FloatExtend => {
                        let value = self.assign_freg()?;
                        self.add_instruction(Opcode::FCvtDS, &[value.clone(), target_value])?;
                        Ok(value)
                    }
                    CastMethod::FloatTrunc => {
                        let value = self.assign_freg()?;
                        self.add_instruction(Opcode::FCvtSD, &[value.clone(), target_value])?;
                        Ok(value)
                    }
                    CastMethod::FloatToSInt => {
                        let value = self.assign_ireg()?;
                        match self.xlen {
                            32 => {
                                self.add_instruction(
                                    if target.borrow().r#type.borrow().is_float_type() {
                                        Opcode::FCvtWS
                                    } else {
                                        Opcode::FCvtWD
                                    },
                                    &[value.clone(), target_value],
                                )?;
                            }
                            64 => {
                                self.add_instruction(
                                    if target.borrow().r#type.borrow().is_float_type() {
                                        Opcode::FCvtLS
                                    } else {
                                        Opcode::FCvtLD
                                    },
                                    &[value.clone(), target_value],
                                )?;
                            }
                            _ => unreachable!(),
                        }
                        Ok(value)
                    }
                    CastMethod::FloatToUInt => {
                        let value = self.assign_ireg()?;
                        match self.xlen {
                            32 => {
                                self.add_instruction(
                                    if target.borrow().r#type.borrow().is_float_type() {
                                        Opcode::FCvtWUS
                                    } else {
                                        Opcode::FCvtWUD
                                    },
                                    &[value.clone(), target_value],
                                )?;
                            }
                            64 => {
                                self.add_instruction(
                                    if target.borrow().r#type.borrow().is_float_type() {
                                        Opcode::FCvtLUS
                                    } else {
                                        Opcode::FCvtLUD
                                    },
                                    &[value.clone(), target_value],
                                )?;
                            }
                            _ => unreachable!(),
                        }
                        Ok(value)
                    }
                    CastMethod::SIntToFloat => {
                        let value = self.assign_freg()?;
                        match self.xlen {
                            32 => {
                                self.add_instruction(
                                    if r#type.borrow().is_float_type() {
                                        Opcode::FCvtSW
                                    } else {
                                        Opcode::FCvtDW
                                    },
                                    &[value.clone(), target_value],
                                )?;
                            }
                            64 => {
                                self.add_instruction(
                                    if r#type.borrow().is_float_type() {
                                        Opcode::FCvtSL
                                    } else {
                                        Opcode::FCvtDL
                                    },
                                    &[value.clone(), target_value],
                                )?;
                            }
                            _ => unreachable!(),
                        }
                        Ok(value)
                    }
                    CastMethod::UIntToFloat => {
                        let value = self.assign_freg()?;
                        match self.xlen {
                            32 => {
                                self.add_instruction(
                                    if r#type.borrow().is_float_type() {
                                        Opcode::FCvtSWU
                                    } else {
                                        Opcode::FCvtDWU
                                    },
                                    &[value.clone(), target_value],
                                )?;
                            }
                            64 => {
                                self.add_instruction(
                                    if r#type.borrow().is_float_type() {
                                        Opcode::FCvtSLU
                                    } else {
                                        Opcode::FCvtDLU
                                    },
                                    &[value.clone(), target_value],
                                )?;
                            }
                            _ => unreachable!(),
                        }
                        Ok(value)
                    }
                    CastMethod::SignedExtend => {
                        let value = self.assign_ireg()?;
                        let size = target.borrow().r#type.borrow().size().unwrap();
                        self.add_instruction(
                            Opcode::LShift,
                            &[
                                value.clone(),
                                target_value,
                                Operand::Immediate((self.xlen - size * 8) as i64),
                            ],
                        )?;
                        self.add_instruction(
                            Opcode::RShiftA,
                            &[
                                value.clone(),
                                value.clone(),
                                Operand::Immediate((self.xlen - size * 8) as i64),
                            ],
                        )?;
                        Ok(value)
                    }
                    CastMethod::ZeroExtand => {
                        let value = self.assign_ireg()?;
                        let size = target.borrow().r#type.borrow().size().unwrap();
                        self.add_instruction(
                            Opcode::LShift,
                            &[
                                value.clone(),
                                target_value,
                                Operand::Immediate((self.xlen - size * 8) as i64),
                            ],
                        )?;
                        self.add_instruction(
                            Opcode::RShiftL,
                            &[
                                value.clone(),
                                value.clone(),
                                Operand::Immediate((self.xlen - size * 8) as i64),
                            ],
                        )?;
                        Ok(value)
                    }
                    CastMethod::IntTrunc => {
                        let value = self.assign_ireg()?;
                        let size = r#type.borrow().size().unwrap();
                        self.add_instruction(
                            Opcode::And,
                            &[
                                value.clone(),
                                target_value,
                                Operand::Immediate((1 << size) - 1),
                            ],
                        )?;
                        Ok(value)
                    }
                    _ => unreachable!(),
                }
            }
            ExprKind::CompoundLiteral {
                storage_classes,
                initializer,
                ..
            } => {
                if storage_classes
                    .iter()
                    .any(|x| x.kind == StorageClassKind::Static)
                {
                    ConstFolder::new().visit_initializer(initializer.clone(), HashMap::new())?;

                    let name = self.add_global(
                        "compoundLiteral",
                        (Some(initializer.borrow().value.clone()), r#type.clone()),
                    )?;
                    let value = Operand::Symbol(name.clone());
                    Ok(value)
                } else {
                    let init_value = self.visit_initializer(initializer, None, &None)?;
                    Ok(init_value)
                }
            }
            ExprKind::FunctionCall { target, arguments } => {
                let xsize = self.xlen / 8;

                let (_, function) = self.functions.get_index_mut(self.cur_function).unwrap();
                let mut frame_size = function.local_frame_size;

                let mut ireg_used = 0;
                let mut freg_used = 0;

                match &get_inner_type(r#type.clone()).borrow().kind {
                    t if t.is_aggregate() => {
                        function.adjust_local_frame_size(
                            r#type.borrow().size().unwrap(),
                            r#type.borrow().align().unwrap(),
                        );
                        frame_size = function.local_frame_size;

                        if t.size().unwrap() > xsize * 2 {
                            self.add_instruction(
                                Opcode::Add,
                                &[A0_REG, FP_REG, Operand::Immediate(-(frame_size as i64))],
                            )?;

                            ireg_used += 1;
                        }
                    }
                    _ => {}
                };

                let mut arg_values = vec![];
                for arg in arguments.iter() {
                    arg_values.push(self.visit_expr(arg)?);
                }

                let mut arg_frame_size = 0;
                for (arg, arg_value) in arguments.iter().zip(arg_values) {
                    let arg_value = self.normalize_to_reg(&arg_value)?;
                    let arg_type = get_inner_type(arg.borrow().r#type.clone());
                    match arg_type.borrow().kind.clone() {
                        t if t.is_float_type() => {
                            if freg_used < 8 {
                                self.add_instruction(
                                    Opcode::FMoveS,
                                    &[Operand::FPReg(10 + freg_used), arg_value],
                                )?;
                                freg_used += 1;
                            } else {
                                self.push_arg(&arg_value, &arg_type, &mut arg_frame_size)?;
                            }
                        }
                        t if t.is_double() => {
                            if freg_used < 8 {
                                self.add_instruction(
                                    Opcode::FMoveD,
                                    &[Operand::FPReg(10 + freg_used), arg_value],
                                )?;
                                freg_used += 1;
                            } else {
                                self.push_arg(&arg_value, &arg_type, &mut arg_frame_size)?;
                            }
                        }
                        //float和double也是scaler, 所以放到上面优先匹配
                        t if t.is_scale() => {
                            if ireg_used < 8 {
                                self.add_instruction(
                                    Opcode::Move,
                                    &[Operand::IntReg(10 + ireg_used), arg_value],
                                )?;
                                ireg_used += 1;
                            } else {
                                self.push_arg(&arg_value, &arg_type, &mut arg_frame_size)?;
                            }
                        }
                        //这时的arg_value应该代表指针
                        t if t.is_aggregate() => {
                            let load_opcode = match self.xlen {
                                32 => Opcode::LoadWU,
                                64 => Opcode::LoadD,
                                _ => unreachable!(),
                            };

                            let size = t.size().unwrap();

                            if size <= xsize * 2 {
                                if ireg_used < 8 {
                                    self.add_instruction(
                                        load_opcode,
                                        &[Operand::IntReg(10 + ireg_used), arg_value.clone()],
                                    )?;
                                    ireg_used += 1;
                                } else {
                                    let value = self.assign_ireg()?;
                                    self.add_instruction(
                                        load_opcode,
                                        &[value.clone(), arg_value.clone()],
                                    )?;
                                    self.push_arg(&value, &arg_type, &mut arg_frame_size)?;
                                }

                                if size > xsize {
                                    self.add_instruction(
                                        Opcode::Add,
                                        &[
                                            arg_value.clone(),
                                            arg_value.clone(),
                                            Operand::Immediate(xsize as i64),
                                        ],
                                    )?;

                                    if ireg_used < 8 {
                                        self.add_instruction(
                                            load_opcode,
                                            &[Operand::IntReg(10 + ireg_used), arg_value],
                                        )?;
                                        ireg_used += 1;
                                    } else {
                                        let value = self.assign_ireg()?;
                                        self.add_instruction(
                                            load_opcode,
                                            &[value.clone(), arg_value],
                                        )?;
                                        self.push_arg(&value, &arg_type, &mut arg_frame_size)?;
                                    }
                                }
                            } else {
                                self.push_arg(&arg_value, &arg_type, &mut arg_frame_size)?;
                            }
                        }
                        _ => {}
                    }
                }

                let (_, function) = self.functions.get_index_mut(self.cur_function).unwrap();
                function.arg_frame_size = function.arg_frame_size.max(arg_frame_size);

                let target_value = self.visit_expr(target)?;
                self.add_instruction(Opcode::Call, &[target_value])?;

                match &get_inner_type(r#type.clone()).borrow().kind {
                    t if t.is_aggregate() && t.size().unwrap() <= xsize * 2 => {
                        //将存在a0,a1中的返回值保存到内存中, 并返回这块内存的起始地址
                        let store_opcode = match self.xlen {
                            32 => Opcode::StoreW,
                            64 => Opcode::StoreD,
                            _ => unreachable!(),
                        };

                        let value = self.assign_ireg()?;
                        self.add_instruction(
                            Opcode::Add,
                            &[
                                value.clone(),
                                FP_REG,
                                Operand::Immediate(-(frame_size as i64)),
                            ],
                        )?;

                        self.add_instruction(
                            store_opcode,
                            &[
                                A0_REG,
                                Operand::Address {
                                    base: Box::new(value.clone()),
                                    offset: 0,
                                },
                            ],
                        )?;

                        if t.size().unwrap() > xsize {
                            self.add_instruction(
                                store_opcode,
                                &[
                                    A1_REG,
                                    Operand::Address {
                                        base: Box::new(value.clone()),
                                        offset: xsize as i64,
                                    },
                                ],
                            )?;
                        }

                        Ok(value)
                    }
                    t if t.is_aggregate() && t.size().unwrap() > xsize * 2 => {
                        //a0存的就是地址
                        let t = self.assign_ireg()?;
                        self.add_instruction(Opcode::Move, &[t.clone(), A0_REG])?;
                        Ok(t)
                    }
                    t if t.is_real_float() => {
                        let t = self.assign_freg()?;
                        self.add_instruction(Opcode::FMoveD, &[t.clone(), FA0_REG])?;
                        Ok(t)
                    }
                    //real float也是scaler, 所以放到上面优先匹配
                    t if t.is_scale() => {
                        let t = self.assign_ireg()?;
                        self.add_instruction(Opcode::Move, &[t.clone(), A0_REG])?;
                        Ok(t)
                    }
                    //随便选的
                    t if t.is_void() => Ok(A0_REG),
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }
}
