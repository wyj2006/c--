use crate::{
    ast::expr::{BinOpKind, CastMethod, Expr, ExprKind, UnaryOpKind},
    codegen::{CodeGen, any_to_basic_type, any_to_basic_value},
    ctype::{RecordKind, TypeKind, cast::remove_qualifier, layout::compute_layout, pointee},
    diagnostic::map_builder_err,
    symtab::{Namespace, SymbolKind},
    variant::Variant,
};
use codespan_reporting::diagnostic::{Diagnostic, Label};
use inkwell::{
    FloatPredicate, IntPredicate,
    values::{AnyValue, AnyValueEnum, InstructionOpcode, IntValue},
};
use std::{cell::RefCell, rc::Rc, u64};

impl<'ctx> CodeGen<'ctx> {
    pub fn visit_expr(
        &mut self,
        node: Rc<RefCell<Expr>>,
    ) -> Result<AnyValueEnum<'ctx>, Diagnostic<usize>> {
        if self.func_values.len() == 0 {
            return match self.to_llvm_value(
                node.borrow().value.clone(),
                Rc::clone(&node.borrow().r#type),
            ) {
                Ok(t) => Ok(t),
                Err(_) => Err(Diagnostic::error()
                    .with_message("not a constant expression")
                    .with_label(Label::primary(node.borrow().file_id, node.borrow().span))),
            };
        }
        let node = node.borrow();
        match &node.kind {
            ExprKind::Name(name) => {
                let symbol = self.lookup(Namespace::Ordinary, &name).unwrap();
                let symbol_type = Rc::clone(&symbol.borrow().r#type);
                match &symbol.borrow().kind {
                    SymbolKind::EnumConst { value } => {
                        Ok(self.to_llvm_value(Variant::Int(value.clone()), symbol_type)?)
                    }
                    _ => Ok(*self.symbol_values.get(&(symbol.as_ptr() as usize)).unwrap()),
                }
            }
            ExprKind::String { .. } => {
                let global_value = self.module.add_global(
                    self.to_llvm_type(Rc::clone(&node.r#type))?
                        .into_array_type(),
                    None,
                    "str",
                );
                global_value.set_initializer(&match any_to_basic_value(
                    self.to_llvm_value(node.value.clone(), Rc::clone(&node.r#type))?,
                ) {
                    Some(t) => t,
                    None => unreachable!(),
                });
                Ok(global_value.as_any_value_enum())
            }
            ExprKind::Char { .. }
            | ExprKind::Integer { .. }
            | ExprKind::Float { .. }
            | ExprKind::True
            | ExprKind::False
            | ExprKind::Nullptr
            | ExprKind::SizeOf { .. }
            | ExprKind::Alignof { .. } => {
                Ok(self.to_llvm_value(node.value.clone(), Rc::clone(&node.r#type))?)
            }
            ExprKind::GenericSelection { assocs, .. } => {
                for assoc in assocs {
                    if assoc.borrow().is_selected {
                        return Ok(self.visit_expr(Rc::clone(&assoc.borrow().expr))?);
                    }
                }
                unreachable!()
            }
            ExprKind::Cast { target, method, .. } => {
                let target_value = if let CastMethod::LToRValue = *method {
                    //避免重复生成target的代码
                    self.context
                        .i32_type()
                        .const_int(0, false)
                        .as_any_value_enum()
                } else {
                    self.visit_expr(Rc::clone(target))?
                };
                match method {
                    CastMethod::Nothing | CastMethod::PtrToPtr => {
                        Ok(self.visit_expr(Rc::clone(target))?)
                    }
                    CastMethod::FuncToPtr => Ok(match self.visit_expr(Rc::clone(target))? {
                        AnyValueEnum::FunctionValue(t) => {
                            //实际上在调用as_any_value_enum后值又变成了FunctionValue
                            t.as_global_value().as_pointer_value().as_any_value_enum()
                        }
                        _ => unreachable!(),
                    }),
                    CastMethod::ArrayToPtr => {
                        Ok(map_builder_err(node.file_id, node.span, unsafe {
                            self.builder.build_in_bounds_gep(
                                self.to_llvm_type(Rc::clone(&target.borrow().r#type))?
                                    .into_array_type(),
                                //target 一般是左值, 而左值一定是PointerValue
                                target_value.into_pointer_value(),
                                &[self.context.i64_type().const_int(0, false)],
                                "",
                            )
                        })?
                        .as_any_value_enum())
                    }
                    CastMethod::LToRValue => Ok(self.build_load(Rc::clone(target))?),
                    _ => Ok(map_builder_err(
                        node.file_id,
                        node.span,
                        self.builder.build_cast(
                            match method {
                                CastMethod::PtrToInt => InstructionOpcode::PtrToInt,
                                CastMethod::IntToPtr => InstructionOpcode::IntToPtr,
                                CastMethod::FloatTrunc => InstructionOpcode::FPTrunc,
                                CastMethod::FloatExtand => InstructionOpcode::FPExt,
                                CastMethod::FloatToSInt => InstructionOpcode::FPToSI,
                                CastMethod::FloatToUInt => InstructionOpcode::FPToUI,
                                CastMethod::SIntToFloat => InstructionOpcode::SIToFP,
                                CastMethod::UIntToFloat => InstructionOpcode::UIToFP,
                                CastMethod::SignedExtand => InstructionOpcode::SExt,
                                CastMethod::ZeroExtand => InstructionOpcode::ZExt,
                                CastMethod::IntTrunc => InstructionOpcode::Trunc,
                                _ => unreachable!(),
                            },
                            match any_to_basic_value(target_value) {
                                Some(t) => t,
                                None => {
                                    return Err(Diagnostic::error()
                                        .with_message(format!(
                                            "cast from a invalid value: {target_value}"
                                        ))
                                        .with_label(Label::primary(
                                            target.borrow().file_id,
                                            target.borrow().span,
                                        )));
                                }
                            },
                            match any_to_basic_type(self.to_llvm_type(Rc::clone(&node.r#type))?) {
                                Some(t) => t,
                                None => {
                                    return Err(Diagnostic::error()
                                        .with_message(format!(
                                            "cast to a invalid type: {}",
                                            node.r#type.borrow()
                                        ))
                                        .with_label(Label::primary(
                                            node.r#type.borrow().file_id,
                                            node.r#type.borrow().span,
                                        )));
                                }
                            },
                            "",
                        ),
                    )?
                    .as_any_value_enum()),
                }
            }
            ExprKind::Conditional {
                condition,
                true_expr,
                false_expr,
            } => {
                let true_block = self
                    .context
                    .append_basic_block(*self.func_values.last().unwrap(), "cond_true");
                let false_block = self
                    .context
                    .append_basic_block(*self.func_values.last().unwrap(), "cond_false");
                let end_block = self
                    .context
                    .append_basic_block(*self.func_values.last().unwrap(), "cond_end");

                let condtition_value = self.to_bool(Rc::clone(condition))?;
                map_builder_err(
                    node.file_id,
                    node.span,
                    self.builder.build_conditional_branch(
                        condtition_value,
                        true_block,
                        false_block,
                    ),
                )?;

                true_block
                    .move_after(self.builder.get_insert_block().unwrap())
                    .unwrap();

                self.builder.position_at_end(true_block);
                let true_value = self.visit_expr(Rc::clone(true_expr))?;
                map_builder_err(
                    node.file_id,
                    node.span,
                    self.builder.build_unconditional_branch(end_block),
                )?;

                false_block
                    .move_after(self.builder.get_insert_block().unwrap())
                    .unwrap();

                self.builder.position_at_end(false_block);
                let false_value = self.visit_expr(Rc::clone(false_expr))?;
                map_builder_err(
                    node.file_id,
                    node.span,
                    self.builder.build_unconditional_branch(end_block),
                )?;

                end_block
                    .move_after(self.builder.get_insert_block().unwrap())
                    .unwrap();

                self.builder.position_at_end(end_block);

                let phi = map_builder_err(
                    node.file_id,
                    node.span,
                    self.builder.build_phi(
                        any_to_basic_type(self.to_llvm_type(Rc::clone(&node.r#type))?).unwrap(),
                        "",
                    ),
                )?;
                phi.add_incoming(&[(&any_to_basic_value(true_value).unwrap(), true_block)]);
                phi.add_incoming(&[(&any_to_basic_value(false_value).unwrap(), false_block)]);

                Ok(phi.as_any_value_enum())
            }
            ExprKind::FunctionCall { target, arguments } => {
                let mut args = Vec::new();
                for arg in arguments {
                    args.push(
                        any_to_basic_value(self.visit_expr(Rc::clone(arg))?)
                            .unwrap()
                            .into(),
                    );
                }

                let target_value = self.visit_expr(Rc::clone(target))?;
                match target_value {
                    AnyValueEnum::PointerValue(t) => Ok(map_builder_err(
                        node.file_id,
                        node.span,
                        self.builder.build_indirect_call(
                            self.to_llvm_type(
                                pointee(Rc::clone(&target.borrow().r#type)).unwrap(),
                            )?
                            .into_function_type(),
                            t,
                            &args,
                            "",
                        ),
                    )?
                    .as_any_value_enum()),
                    AnyValueEnum::FunctionValue(t) => Ok(map_builder_err(
                        node.file_id,
                        node.span,
                        self.builder.build_direct_call(t, &args, ""),
                    )?
                    .as_any_value_enum()),
                    _ => unreachable!(),
                }
            }
            ExprKind::Subscript { target, index } => {
                let target_value = self.visit_expr(Rc::clone(target))?;
                let index_value = self.visit_expr(Rc::clone(index))?;
                match (target_value, index_value) {
                    (AnyValueEnum::PointerValue(target), AnyValueEnum::IntValue(index))
                    | (AnyValueEnum::IntValue(index), AnyValueEnum::PointerValue(target)) => {
                        Ok(map_builder_err(node.file_id, node.span, unsafe {
                            self.builder.build_in_bounds_gep(
                                any_to_basic_type(self.to_llvm_type(Rc::clone(&node.r#type))?)
                                    .unwrap(),
                                target,
                                &[index],
                                "",
                            )
                        })?
                        .as_any_value_enum())
                    }
                    _ => unreachable!(),
                }
            }
            ExprKind::UnaryOp { op, operand } => {
                let operand_value = if let UnaryOpKind::Not = op {
                    self.to_bool(Rc::clone(operand))?.as_any_value_enum()
                } else {
                    self.visit_expr(Rc::clone(operand))?
                };
                match op {
                    UnaryOpKind::AddressOf => match &operand.borrow().kind {
                        ExprKind::UnaryOp {
                            op: UnaryOpKind::Dereference,
                            operand,
                        } => Ok(self.visit_expr(Rc::clone(operand))?),
                        _ => match operand_value {
                            AnyValueEnum::FunctionValue(t) => {
                                Ok(t.as_global_value().as_pointer_value().as_any_value_enum())
                            }
                            //对于左值, operand_value一定是指针
                            _ => Ok(operand_value),
                        },
                    },
                    UnaryOpKind::Dereference => Ok(map_builder_err(
                        node.file_id,
                        node.span,
                        self.builder.build_load(
                            self.to_llvm_type(Rc::clone(&operand.borrow().r#type))?
                                .into_pointer_type(),
                            operand_value.into_pointer_value(),
                            "",
                        ),
                    )?
                    .as_any_value_enum()),
                    UnaryOpKind::Not => Ok(map_builder_err(
                        node.file_id,
                        node.span,
                        self.builder.build_int_compare(
                            IntPredicate::EQ,
                            operand_value.into_int_value(),
                            self.context.bool_type().const_int(0, false),
                            "",
                        ),
                    )?
                    .as_any_value_enum()),
                    UnaryOpKind::Positive => Ok(operand_value),
                    UnaryOpKind::Negative => {
                        match &operand.borrow().r#type.borrow().kind {
                            t if t.is_integer() => Ok(map_builder_err(
                                node.file_id,
                                node.span,
                                self.builder
                                    .build_int_neg(operand_value.into_int_value(), ""),
                            )?
                            .as_any_value_enum()),
                            t if t.is_real_float() => Ok(map_builder_err(
                                node.file_id,
                                node.span,
                                self.builder
                                    .build_float_neg(operand_value.into_float_value(), ""),
                            )?
                            .as_any_value_enum()),
                            //TODO 复数和十进制浮点数
                            _ => todo!(),
                        }
                    }
                    UnaryOpKind::BitNot => Ok(map_builder_err(
                        node.file_id,
                        node.span,
                        self.builder.build_not(operand_value.into_int_value(), ""),
                    )?
                    .as_any_value_enum()),
                    UnaryOpKind::PostfixDec
                    | UnaryOpKind::PrefixDec
                    | UnaryOpKind::PostfixInc
                    | UnaryOpKind::PrefixInc => {
                        let offset: i64 = match op {
                            UnaryOpKind::PostfixDec | UnaryOpKind::PrefixDec => -1,
                            UnaryOpKind::PrefixInc | UnaryOpKind::PostfixInc => 1,
                            _ => unreachable!(),
                        };
                        //操作数是左值且没有进行过左值转换
                        let old_value = map_builder_err(
                            node.file_id,
                            node.span,
                            self.builder.build_load(
                                any_to_basic_type(
                                    self.to_llvm_type(Rc::clone(&operand.borrow().r#type))?,
                                )
                                .unwrap(),
                                operand_value.into_pointer_value(),
                                "",
                            ),
                        )?;
                        let new_value = match &operand.borrow().r#type.borrow().kind {
                            t if t.is_integer() => map_builder_err(
                                node.file_id,
                                node.span,
                                self.builder.build_int_add(
                                    old_value.into_int_value(),
                                    old_value
                                        .get_type()
                                        .into_int_type()
                                        .const_int(offset as u64, true),
                                    "",
                                ),
                            )?
                            .as_any_value_enum(),
                            t if t.is_real_float() => map_builder_err(
                                node.file_id,
                                node.span,
                                self.builder.build_float_add(
                                    old_value.into_float_value(),
                                    old_value
                                        .get_type()
                                        .into_float_type()
                                        .const_float(offset as f64),
                                    "",
                                ),
                            )?
                            .as_any_value_enum(),
                            t if t.is_pointer() => {
                                map_builder_err(node.file_id, node.span, unsafe {
                                    self.builder.build_in_bounds_gep(
                                        match any_to_basic_type(self.to_llvm_type(
                                            pointee(Rc::clone(&operand.borrow().r#type)).unwrap(),
                                        )?) {
                                            Some(t) => t,
                                            None => {
                                                return Err(Diagnostic::error()
                                                    .with_message(format!(
                                                        "not point to a basic type"
                                                    ))
                                                    .with_label(Label::primary(
                                                        operand.borrow().file_id,
                                                        operand.borrow().span,
                                                    )));
                                            }
                                        },
                                        old_value.into_pointer_value(),
                                        &[self.context.i64_type().const_int(offset as u64, true)],
                                        "",
                                    )
                                })?
                                .as_any_value_enum()
                            }
                            _ => unreachable!(),
                        };
                        map_builder_err(
                            node.file_id,
                            node.span,
                            self.builder.build_store(
                                //操作数本身就是左值
                                operand_value.into_pointer_value(),
                                any_to_basic_value(new_value).unwrap(),
                            ),
                        )?;
                        match op {
                            UnaryOpKind::PostfixInc | UnaryOpKind::PostfixDec => {
                                Ok(old_value.as_any_value_enum())
                            }
                            UnaryOpKind::PrefixInc | UnaryOpKind::PrefixDec => Ok(new_value),
                            _ => unreachable!(),
                        }
                    }
                }
            }
            ExprKind::BinOp { op, left, right } => match op {
                BinOpKind::And => {
                    let left_block = self.builder.get_insert_block().unwrap();
                    let right_block = self
                        .context
                        .append_basic_block(*self.func_values.last().unwrap(), "and_right");
                    let end_block = self
                        .context
                        .append_basic_block(*self.func_values.last().unwrap(), "and_end");

                    let left_value = self.to_bool(Rc::clone(left))?;
                    map_builder_err(
                        node.file_id,
                        node.span,
                        self.builder
                            .build_conditional_branch(left_value, right_block, end_block),
                    )?;

                    right_block
                        .move_after(self.builder.get_insert_block().unwrap())
                        .unwrap();

                    self.builder.position_at_end(right_block);
                    let right_value = self.to_bool(Rc::clone(right))?;
                    map_builder_err(
                        node.file_id,
                        node.span,
                        self.builder.build_unconditional_branch(end_block),
                    )?;

                    end_block
                        .move_after(self.builder.get_insert_block().unwrap())
                        .unwrap();

                    self.builder.position_at_end(end_block);
                    let phi = map_builder_err(
                        node.file_id,
                        node.span,
                        self.builder.build_phi(
                            any_to_basic_type(self.to_llvm_type(Rc::clone(&node.r#type))?).unwrap(),
                            "",
                        ),
                    )?;
                    phi.add_incoming(&[(&self.context.i32_type().const_int(0, false), left_block)]);
                    phi.add_incoming(&[(&right_value, right_block)]);

                    Ok(phi.as_any_value_enum())
                }
                BinOpKind::Or => {
                    let left_block = self.builder.get_insert_block().unwrap();
                    let right_block = self
                        .context
                        .append_basic_block(*self.func_values.last().unwrap(), "or_right");
                    let end_block = self
                        .context
                        .append_basic_block(*self.func_values.last().unwrap(), "or_end");

                    let left_value = self.to_bool(Rc::clone(left))?;
                    map_builder_err(
                        node.file_id,
                        node.span,
                        self.builder
                            .build_conditional_branch(left_value, end_block, right_block),
                    )?;

                    right_block
                        .move_after(self.builder.get_insert_block().unwrap())
                        .unwrap();

                    self.builder.position_at_end(right_block);
                    let right_value = self.to_bool(Rc::clone(right))?;
                    map_builder_err(
                        node.file_id,
                        node.span,
                        self.builder.build_unconditional_branch(end_block),
                    )?;

                    end_block
                        .move_after(self.builder.get_insert_block().unwrap())
                        .unwrap();

                    self.builder.position_at_end(end_block);
                    let phi = map_builder_err(
                        node.file_id,
                        node.span,
                        self.builder.build_phi(
                            any_to_basic_type(self.to_llvm_type(Rc::clone(&node.r#type))?).unwrap(),
                            "",
                        ),
                    )?;
                    phi.add_incoming(&[(&self.context.i32_type().const_int(1, false), left_block)]);
                    phi.add_incoming(&[(&right_value, right_block)]);

                    Ok(phi.as_any_value_enum())
                }
                BinOpKind::Ge | BinOpKind::Gt | BinOpKind::Le | BinOpKind::Lt => {
                    let left_value = self.visit_expr(Rc::clone(left))?;
                    let right_value = self.visit_expr(Rc::clone(right))?;

                    match (
                        &left.borrow().r#type.borrow().kind,
                        &right.borrow().r#type.borrow().kind,
                    ) {
                        (a, b) if a.is_integer() && b.is_integer() => Ok(map_builder_err(
                            node.file_id,
                            node.span,
                            self.builder.build_int_compare(
                                match op {
                                    //进行常用算术转换后a和b的符号应该一样
                                    BinOpKind::Ge => {
                                        if a.is_unsigned().unwrap() {
                                            IntPredicate::UGE
                                        } else {
                                            IntPredicate::SGE
                                        }
                                    }
                                    BinOpKind::Gt => {
                                        if a.is_unsigned().unwrap() {
                                            IntPredicate::UGT
                                        } else {
                                            IntPredicate::SGT
                                        }
                                    }
                                    BinOpKind::Le => {
                                        if a.is_unsigned().unwrap() {
                                            IntPredicate::ULE
                                        } else {
                                            IntPredicate::SLE
                                        }
                                    }
                                    BinOpKind::Lt => {
                                        if a.is_unsigned().unwrap() {
                                            IntPredicate::ULT
                                        } else {
                                            IntPredicate::SLT
                                        }
                                    }
                                    _ => unreachable!(),
                                },
                                left_value.into_int_value(),
                                right_value.into_int_value(),
                                "",
                            ),
                        )?
                        .as_any_value_enum()),
                        (a, b) if a.is_real_float() && b.is_real_float() => Ok(map_builder_err(
                            node.file_id,
                            node.span,
                            self.builder.build_float_compare(
                                match op {
                                    BinOpKind::Ge => FloatPredicate::OGE,
                                    BinOpKind::Gt => FloatPredicate::OGT,
                                    BinOpKind::Le => FloatPredicate::OLE,
                                    BinOpKind::Lt => FloatPredicate::OLT,
                                    _ => unreachable!(),
                                },
                                left_value.into_float_value(),
                                right_value.into_float_value(),
                                "",
                            ),
                        )?
                        .as_any_value_enum()),
                        (a, b) if a.is_pointer() && b.is_pointer() => Ok(map_builder_err(
                            node.file_id,
                            node.span,
                            self.builder.build_int_compare(
                                match op {
                                    BinOpKind::Ge => IntPredicate::UGE,
                                    BinOpKind::Gt => IntPredicate::UGT,
                                    BinOpKind::Le => IntPredicate::ULE,
                                    BinOpKind::Lt => IntPredicate::ULT,
                                    _ => unreachable!(),
                                },
                                left_value.into_pointer_value(),
                                right_value.into_pointer_value(),
                                "",
                            ),
                        )?
                        .as_any_value_enum()),
                        _ => unreachable!(),
                    }
                }
                BinOpKind::Eq | BinOpKind::Neq => {
                    let left_value = self.visit_expr(Rc::clone(left))?;
                    let right_value = self.visit_expr(Rc::clone(right))?;

                    match (
                        &left.borrow().r#type.borrow().kind,
                        &right.borrow().r#type.borrow().kind,
                    ) {
                        (a, b) if a.is_integer() && b.is_integer() => Ok(map_builder_err(
                            node.file_id,
                            node.span,
                            self.builder.build_int_compare(
                                match op {
                                    BinOpKind::Eq => IntPredicate::EQ,
                                    BinOpKind::Neq => IntPredicate::NE,
                                    _ => unreachable!(),
                                },
                                left_value.into_int_value(),
                                right_value.into_int_value(),
                                "",
                            ),
                        )?
                        .as_any_value_enum()),
                        (a, b) if a.is_real_float() && b.is_real_float() => Ok(map_builder_err(
                            node.file_id,
                            node.span,
                            self.builder.build_float_compare(
                                match op {
                                    BinOpKind::Eq => FloatPredicate::OEQ,
                                    BinOpKind::Neq => FloatPredicate::ONE,
                                    _ => unreachable!(),
                                },
                                left_value.into_float_value(),
                                right_value.into_float_value(),
                                "",
                            ),
                        )?
                        .as_any_value_enum()),
                        (a, b)
                            if (a.is_pointer() || a.is_nullptr())
                                && (b.is_pointer() || b.is_nullptr()) =>
                        {
                            Ok(map_builder_err(
                                node.file_id,
                                node.span,
                                self.builder.build_int_compare(
                                    match op {
                                        BinOpKind::Eq => IntPredicate::EQ,
                                        BinOpKind::Neq => IntPredicate::NE,
                                        _ => unreachable!(),
                                    },
                                    left_value.into_pointer_value(),
                                    right_value.into_pointer_value(),
                                    "",
                                ),
                            )?
                            .as_any_value_enum())
                        }
                        //TODO 复数和十进制浮点数
                        _ => todo!(),
                    }
                }
                BinOpKind::Add => {
                    let left_value = self.visit_expr(Rc::clone(left))?;
                    let right_value = self.visit_expr(Rc::clone(right))?;
                    match (
                        &left.borrow().r#type.borrow().kind,
                        &right.borrow().r#type.borrow().kind,
                    ) {
                        (a, b) if a.is_integer() && b.is_integer() => Ok(map_builder_err(
                            node.file_id,
                            node.span,
                            self.builder.build_int_add(
                                left_value.into_int_value(),
                                right_value.into_int_value(),
                                "",
                            ),
                        )?
                        .as_any_value_enum()),
                        (a, b) if a.is_real_float() && b.is_real_float() => Ok(map_builder_err(
                            node.file_id,
                            node.span,
                            self.builder.build_float_add(
                                left_value.into_float_value(),
                                right_value.into_float_value(),
                                "",
                            ),
                        )?
                        .as_any_value_enum()),
                        (TypeKind::Pointer(pointee), b) if b.is_integer() => {
                            Ok(map_builder_err(node.file_id, node.span, unsafe {
                                self.builder.build_in_bounds_gep(
                                    any_to_basic_type(self.to_llvm_type(Rc::clone(pointee))?)
                                        .unwrap(),
                                    left_value.into_pointer_value(),
                                    &[right_value.into_int_value()],
                                    "",
                                )
                            })?
                            .as_any_value_enum())
                        }
                        (a, TypeKind::Pointer(pointee)) if a.is_integer() => {
                            Ok(map_builder_err(node.file_id, node.span, unsafe {
                                self.builder.build_in_bounds_gep(
                                    any_to_basic_type(self.to_llvm_type(Rc::clone(pointee))?)
                                        .unwrap(),
                                    right_value.into_pointer_value(),
                                    &[left_value.into_int_value()],
                                    "",
                                )
                            })?
                            .as_any_value_enum())
                        }
                        //TODO 复数和十进制浮点数
                        _ => todo!(),
                    }
                }
                BinOpKind::Sub => {
                    let left_value = self.visit_expr(Rc::clone(left))?;
                    let right_value = self.visit_expr(Rc::clone(right))?;
                    match (
                        &left.borrow().r#type.borrow().kind,
                        &right.borrow().r#type.borrow().kind,
                    ) {
                        (a, b) if a.is_integer() && b.is_integer() => Ok(map_builder_err(
                            node.file_id,
                            node.span,
                            self.builder.build_int_sub(
                                left_value.into_int_value(),
                                right_value.into_int_value(),
                                "",
                            ),
                        )?
                        .as_any_value_enum()),
                        (a, b) if a.is_real_float() && b.is_real_float() => Ok(map_builder_err(
                            node.file_id,
                            node.span,
                            self.builder.build_float_sub(
                                left_value.into_float_value(),
                                right_value.into_float_value(),
                                "",
                            ),
                        )?
                        .as_any_value_enum()),
                        (TypeKind::Pointer(pointee), b) if b.is_integer() => {
                            let right_neg = map_builder_err(
                                node.file_id,
                                node.span,
                                self.builder.build_int_neg(right_value.into_int_value(), ""),
                            )?;
                            Ok(map_builder_err(node.file_id, node.span, unsafe {
                                self.builder.build_in_bounds_gep(
                                    any_to_basic_type(self.to_llvm_type(Rc::clone(pointee))?)
                                        .unwrap(),
                                    left_value.into_pointer_value(),
                                    &[right_neg],
                                    "",
                                )
                            })?
                            .as_any_value_enum())
                        }
                        //TODO 复数和十进制浮点数
                        _ => todo!(),
                    }
                }
                BinOpKind::Mul | BinOpKind::Div | BinOpKind::Mod => {
                    let left_value = self.visit_expr(Rc::clone(left))?;
                    let right_value = self.visit_expr(Rc::clone(right))?;
                    match (
                        &left.borrow().r#type.borrow().kind,
                        &right.borrow().r#type.borrow().kind,
                    ) {
                        (a, b) if a.is_integer() && b.is_integer() => Ok(map_builder_err(
                            node.file_id,
                            node.span,
                            match op {
                                BinOpKind::Mul => self.builder.build_int_mul(
                                    left_value.into_int_value(),
                                    right_value.into_int_value(),
                                    "",
                                ),
                                //经过常用算术转换后a和b的符号应该相同
                                BinOpKind::Div => {
                                    if a.is_unsigned().unwrap() {
                                        self.builder.build_int_unsigned_div(
                                            left_value.into_int_value(),
                                            right_value.into_int_value(),
                                            "",
                                        )
                                    } else {
                                        self.builder.build_int_signed_div(
                                            left_value.into_int_value(),
                                            right_value.into_int_value(),
                                            "",
                                        )
                                    }
                                }
                                BinOpKind::Mod => {
                                    if a.is_unsigned().unwrap() {
                                        self.builder.build_int_unsigned_rem(
                                            left_value.into_int_value(),
                                            right_value.into_int_value(),
                                            "",
                                        )
                                    } else {
                                        self.builder.build_int_signed_rem(
                                            left_value.into_int_value(),
                                            right_value.into_int_value(),
                                            "",
                                        )
                                    }
                                }
                                _ => unreachable!(),
                            },
                        )?
                        .as_any_value_enum()),
                        (a, b) if a.is_real_float() && b.is_real_float() => Ok(map_builder_err(
                            node.file_id,
                            node.span,
                            match op {
                                BinOpKind::Mul => self.builder.build_float_mul(
                                    left_value.into_float_value(),
                                    right_value.into_float_value(),
                                    "",
                                ),
                                BinOpKind::Div => self.builder.build_float_div(
                                    left_value.into_float_value(),
                                    right_value.into_float_value(),
                                    "",
                                ),
                                _ => unreachable!(),
                            },
                        )?
                        .as_any_value_enum()),
                        //TODO 复数和十进制浮点数
                        _ => todo!(),
                    }
                }
                BinOpKind::BitAnd
                | BinOpKind::BitOr
                | BinOpKind::BitXOr
                | BinOpKind::LShift
                | BinOpKind::RShift => {
                    let left_value = self.visit_expr(Rc::clone(left))?;
                    let right_value = self.visit_expr(Rc::clone(right))?;
                    match (
                        &left.borrow().r#type.borrow().kind,
                        &right.borrow().r#type.borrow().kind,
                    ) {
                        (a, b) if a.is_integer() && b.is_integer() => Ok(map_builder_err(
                            node.file_id,
                            node.span,
                            match op {
                                BinOpKind::BitAnd => self.builder.build_and(
                                    left_value.into_int_value(),
                                    right_value.into_int_value(),
                                    "",
                                ),
                                BinOpKind::BitOr => self.builder.build_or(
                                    left_value.into_int_value(),
                                    right_value.into_int_value(),
                                    "",
                                ),
                                BinOpKind::BitXOr => self.builder.build_xor(
                                    left_value.into_int_value(),
                                    right_value.into_int_value(),
                                    "",
                                ),
                                BinOpKind::LShift => self.builder.build_left_shift(
                                    left_value.into_int_value(),
                                    right_value.into_int_value(),
                                    "",
                                ),
                                BinOpKind::RShift => self.builder.build_right_shift(
                                    left_value.into_int_value(),
                                    right_value.into_int_value(),
                                    !a.is_unsigned().unwrap(),
                                    "",
                                ),
                                _ => unreachable!(),
                            },
                        )?
                        .as_any_value_enum()),
                        _ => unreachable!(),
                    }
                }
                BinOpKind::Assign => {
                    let right_value = self.visit_expr(Rc::clone(right))?;
                    self.build_store(Rc::clone(left), right_value)?;
                    Ok(right_value)
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
                        ..Expr::new(
                            node.file_id,
                            node.span,
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
                    let right_value = self.visit_expr(eq_expr)?;
                    self.build_store(Rc::clone(left), right_value)?;
                    Ok(right_value)
                }
                BinOpKind::Comma => {
                    self.visit_expr(Rc::clone(left))?;
                    let right_value = self.visit_expr(Rc::clone(right))?;
                    Ok(right_value)
                }
            },
            //如果 is_arrow==true 那么target的值是指针, 如果是false, 因为不会进行左值转换, 所以target的值还是指针
            ExprKind::MemberAccess {
                target,
                name,
                is_arrow,
                ..
            } => {
                let target_value = self.visit_expr(Rc::clone(target))?;

                let target_type = if *is_arrow {
                    pointee(Rc::clone(&target.borrow().r#type)).unwrap()
                } else {
                    Rc::clone(&target.borrow().r#type)
                };

                let record_type = remove_qualifier(Rc::clone(&target_type));

                match &record_type.borrow().kind {
                    TypeKind::Record {
                        kind: RecordKind::Struct,
                        members: Some(members),
                        ..
                    } => {
                        let SymbolKind::Member { index, .. } =
                            &members.get(name).unwrap().borrow().kind
                        else {
                            unreachable!();
                        };
                        Ok(map_builder_err(
                            node.file_id,
                            node.span,
                            self.builder.build_struct_gep(
                                any_to_basic_type(self.to_llvm_type(Rc::clone(&record_type))?)
                                    .unwrap(),
                                target_value.into_pointer_value(),
                                *index as u32,
                                "",
                            ),
                        )?
                        .as_any_value_enum())
                    }
                    TypeKind::Record {
                        kind: RecordKind::Union,
                        ..
                    } => Ok(target_value.as_any_value_enum()),
                    _ => unreachable!(),
                }
            }
            ExprKind::CompoundLiteral { .. } => {
                todo!()
            }
        }
    }

    //将标量类型的表达式转换为bool值
    pub fn to_bool(
        &mut self,
        expr: Rc<RefCell<Expr>>,
    ) -> Result<IntValue<'ctx>, Diagnostic<usize>> {
        let file_id = expr.borrow().file_id;
        let span = expr.borrow().span;
        let value = self.visit_expr(Rc::clone(&expr))?;
        match &expr.borrow().r#type.borrow().kind {
            t if t.is_integer() => Ok(map_builder_err(
                file_id,
                span,
                self.builder.build_int_compare(
                    IntPredicate::NE,
                    value.into_int_value(),
                    value.get_type().into_int_type().const_int(0, false),
                    "",
                ),
            )?),
            t if t.is_real_float() => Ok(map_builder_err(
                file_id,
                span,
                self.builder.build_float_compare(
                    FloatPredicate::ONE,
                    value.into_float_value(),
                    value.get_type().into_float_type().const_float(0.),
                    "",
                ),
            )?),
            t if t.is_pointer() => Ok(map_builder_err(
                file_id,
                span,
                self.builder.build_int_compare(
                    IntPredicate::NE,
                    value.into_pointer_value(),
                    value.get_type().into_pointer_type().const_null(),
                    "",
                ),
            )?),
            t if t.is_nullptr() => Ok(self.context.i32_type().const_int(0, false)),
            //TODO 复数和十进制浮点数
            _ => todo!(),
        }
    }

    //如果expr是位域访问, 那么生成加载位域值的操作, 否则就是正常的load代码
    pub fn build_load(
        &mut self,
        node: Rc<RefCell<Expr>>,
    ) -> Result<AnyValueEnum<'ctx>, Diagnostic<usize>> {
        let value = self.visit_expr(Rc::clone(&node))?;
        let value = map_builder_err(
            node.borrow().file_id,
            node.borrow().span,
            self.builder.build_load(
                match any_to_basic_type(self.to_llvm_type(Rc::clone(&node.borrow().r#type))?) {
                    Some(t) => t,
                    None => {
                        return Err(Diagnostic::error()
                            .with_message(format!(
                                "invalid type for lvalue: {}",
                                node.borrow().r#type.borrow()
                            ))
                            .with_label(Label::primary(
                                node.borrow().r#type.borrow().file_id,
                                node.borrow().r#type.borrow().span,
                            )));
                    }
                },
                value.into_pointer_value(),
                "",
            ),
        )?
        .as_any_value_enum();

        match &*node.borrow() {
            Expr {
                file_id,
                span,
                kind:
                    ExprKind::MemberAccess {
                        target,
                        is_arrow,
                        name,
                    },
                symbol: Some(symbol),
                ..
            } => {
                let target_type = if *is_arrow {
                    pointee(Rc::clone(&target.borrow().r#type)).unwrap()
                } else {
                    Rc::clone(&target.borrow().r#type)
                };
                let record_type = remove_qualifier(Rc::clone(&target_type));

                if let TypeKind::Record { .. } = record_type.borrow().kind {
                    let layout = compute_layout(Rc::clone(&record_type)).unwrap();

                    let SymbolKind::Member { index, .. } = &symbol.borrow().kind else {
                        unreachable!();
                    };

                    let layout = &layout.children[*index];
                    for child in &layout.children {
                        if let Some(t) = &child.name
                            && *t == *name
                        {
                            let mask = self
                                .context
                                .i32_type()
                                .const_int((1 << child.width) - 1, false);
                            let lshr = map_builder_err(
                                *file_id,
                                *span,
                                self.builder.build_right_shift(
                                    value.into_int_value(),
                                    self.context
                                        .i32_type()
                                        .const_int(child.offset as u64, false),
                                    false,
                                    "lshr",
                                ),
                            )?;
                            return Ok(map_builder_err(
                                *file_id,
                                *span,
                                self.builder.build_and(lshr, mask, ""),
                            )?
                            .as_any_value_enum());
                        }
                    }
                }
            }
            _ => {}
        }

        Ok(value)
    }

    //行为类似于build_load
    pub fn build_store(
        &mut self,
        node: Rc<RefCell<Expr>>,
        mut value: AnyValueEnum<'ctx>,
    ) -> Result<(), Diagnostic<usize>> {
        let ptr = self.visit_expr(Rc::clone(&node))?.into_pointer_value();

        match &*node.borrow() {
            Expr {
                file_id,
                span,
                kind:
                    ExprKind::MemberAccess {
                        target,
                        is_arrow,
                        name,
                    },
                symbol: Some(symbol),
                ..
            } => {
                let target_type = if *is_arrow {
                    pointee(Rc::clone(&target.borrow().r#type)).unwrap()
                } else {
                    Rc::clone(&target.borrow().r#type)
                };
                let record_type = remove_qualifier(Rc::clone(&target_type));
                if let TypeKind::Record { .. } = record_type.borrow().kind {
                    let layout = compute_layout(Rc::clone(&record_type)).unwrap();

                    let SymbolKind::Member { index, .. } = &symbol.borrow().kind else {
                        unreachable!();
                    };

                    let layout = &layout.children[*index];
                    for child in &layout.children {
                        if let Some(t) = &child.name
                            && *t == *name
                        {
                            let mask = self
                                .context
                                .i32_type()
                                .const_int(!(((1 << child.width) - 1) << child.offset), false);

                            let old_value = map_builder_err(
                                *file_id,
                                *span,
                                self.builder.build_load(
                                    any_to_basic_type(
                                        self.to_llvm_type(Rc::clone(&node.borrow().r#type))?,
                                    )
                                    .unwrap(),
                                    ptr,
                                    "load",
                                ),
                            )?;
                            let masked_value = map_builder_err(
                                *file_id,
                                *span,
                                self.builder.build_and(
                                    old_value.into_int_value(),
                                    mask,
                                    "and_mask",
                                ),
                            )?;
                            let shl = map_builder_err(
                                *file_id,
                                *span,
                                self.builder.build_left_shift(
                                    value.into_int_value(),
                                    self.context
                                        .i32_type()
                                        .const_int(child.offset as u64, false),
                                    "shl",
                                ),
                            )?;
                            value = map_builder_err(
                                *file_id,
                                *span,
                                self.builder.build_or(shl, masked_value, "or"),
                            )?
                            .as_any_value_enum();
                            break;
                        }
                    }
                }
            }
            _ => {}
        }

        map_builder_err(
            node.borrow().file_id,
            node.borrow().span,
            self.builder
                .build_store(ptr, any_to_basic_value(value).unwrap()),
        )?;
        Ok(())
    }
}
