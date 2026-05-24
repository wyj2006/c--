use crate::{
    ast::expr::{BinOpKind, CastMethod, Expr, ExprKind, UnaryOpKind},
    codegen::{CodeGen, builder::Builder},
    ctype::{
        TypeKind,
        cast::remove_qualifier,
        layout::{ConstDesignation, compute_layout},
        pointee,
    },
    optimizer::constfolder::ConstFolder,
    symtab::SymbolKind,
    variant::Variant,
};
use codespan_reporting::diagnostic::Diagnostic;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

impl<B: Builder> CodeGen<B> {
    pub fn visit_expr(&mut self, node: &Rc<RefCell<Expr>>) -> Result<B::Value, Diagnostic<usize>> {
        //如果没有副作用, 尝试直接计算出它的值
        if !node.borrow().has_side_effects {
            //TODO 避免重复调用ConstFolder
            ConstFolder::new().visit_expr(Rc::clone(node), HashMap::new())?;

            self.builder
                .append_context(node.borrow().file_id, node.borrow().span);

            if let Ok(t) = self
                .builder
                .variant_to_value(&node.borrow().value, &node.borrow().r#type)
            {
                self.builder.pop_context();
                return Ok(t);
            }

            self.builder.pop_context();
        }

        let Expr {
            file_id,
            span,
            kind,
            r#type,
            value:_,
            is_lvalue: _,
            symbol,
            has_side_effects:_,
        } = &*node.borrow();

        self.builder.append_context(*file_id, *span);

        let result;

        match kind {
            ExprKind::Name(name) => {
                let Some(symbol) = symbol else { unreachable!() };
                match &symbol.borrow().kind {
                    SymbolKind::EnumConst { value } => {
                        result = self
                            .builder
                            .variant_to_value(&Variant::Int(value.clone()), r#type)?;
                    }
                    _ => result = self.builder.load_var(name)?,
                }
            }
            ExprKind::GenericSelection { assocs, .. } => {
                let assoc = assocs.iter().find(|&x| x.borrow().is_selected).unwrap();
                result = self.visit_expr(&assoc.borrow().expr)?;
            }
            ExprKind::Cast { target, method, .. } => match method {
                CastMethod::LToRValue => {
                    result = self.build_load(target)?;
                }
                _ => {
                    let target_value = self.visit_expr(target)?;
                    result = self.builder.cast(&target_value, r#type, *method)?;
                }
            },
            ExprKind::Subscript { target, index } => {
                let target_value = self.visit_expr(target)?;
                let index_value = self.visit_expr(index)?;
                result = self
                    .builder
                    .subscript(&target_value, &index_value, r#type)?;
            }
            ExprKind::MemberAccess {
                target,
                name,
                is_arrow,
            } => {
                let target_value = self.visit_expr(target)?;
                let target_type = if *is_arrow {
                    pointee(Rc::clone(&target.borrow().r#type)).unwrap()
                } else {
                    Rc::clone(&target.borrow().r#type)
                };

                let record_type = remove_qualifier(Rc::clone(&target_type));
                result = self
                    .builder
                    .load_member(&target_value, &record_type, name)?;
            }
            ExprKind::FunctionCall { target, arguments } => {
                let function = self.visit_expr(target)?;
                let mut args = vec![];
                for arg in arguments {
                    args.push(self.visit_expr(arg)?);
                }
                result = self.builder.call(&function, &args, r#type)?;
            }
            ExprKind::CompoundLiteral {
                storage_classes,
                initializer,
                ..
            } => {
                let init_value = self.visit_initializer(initializer)?;
                result =
                    self.builder
                        .load_compound_literal(r#type, storage_classes, &init_value)?;
            }
            ExprKind::Conditional {
                condition,
                true_expr,
                false_expr,
            } => {
                let cond_block = self.builder.current_basic_block()?;
                let true_block = self.builder.append_basic_block("cond_true")?;
                let false_block = self.builder.append_basic_block("cond_false")?;
                let end_block = self.builder.append_basic_block("cond_end")?;

                self.builder.position_at_end(&cond_block);
                let cond_value = self.visit_expr(condition)?;
                self.builder
                    .conditional_branch(&cond_value, &true_block, &false_block)?;

                self.builder.position_at_end(&true_block);
                let true_value = self.visit_expr(true_expr)?;
                self.builder.unconditional_branch(&end_block)?;

                self.builder.position_at_end(&false_block);
                let false_value = self.visit_expr(false_expr)?;
                self.builder.unconditional_branch(&end_block)?;

                self.builder.position_at_end(&end_block);
                result = self.builder.phi(
                    r#type,
                    &[(true_value, true_block), (false_value, false_block)],
                )?;
            }
            ExprKind::UnaryOp { op, operand } => {
                let operand_value = self.visit_expr(operand)?;
                result = match op {
                    UnaryOpKind::AddressOf => match &operand.borrow().kind {
                        ExprKind::UnaryOp {
                            op: UnaryOpKind::Dereference,
                            operand,
                        } => self.visit_expr(operand)?,
                        _ => self
                            .builder
                            .unaryop(*op, &operand_value, &operand.borrow().r#type)?,
                    },
                    _ => self
                        .builder
                        .unaryop(*op, &operand_value, &operand.borrow().r#type)?,
                }
            }
            ExprKind::BinOp { op, left, right } => match op {
                BinOpKind::And => {
                    let pre_block = self.builder.current_basic_block()?;
                    let left_block = self.builder.append_basic_block("and_left")?;
                    let right_block = self.builder.append_basic_block("and_right")?;
                    let end_block = self.builder.append_basic_block("and_end")?;

                    self.builder.position_at_end(&pre_block);
                    self.builder.unconditional_branch(&left_block)?;

                    self.builder.position_at_end(&left_block);
                    let left_value = self.visit_expr(left)?;
                    self.builder
                        .conditional_branch(&left_value, &right_block, &end_block)?;

                    self.builder.position_at_end(&right_block);
                    let right_value = self.visit_expr(right)?;
                    self.builder.unconditional_branch(&end_block)?;

                    self.builder.position_at_end(&end_block);
                    result = self.builder.phi(
                        r#type,
                        &[(left_value, left_block), (right_value, right_block)],
                    )?;
                }
                BinOpKind::Or => {
                    let pre_block = self.builder.current_basic_block()?;
                    let left_block = self.builder.append_basic_block("or_left")?;
                    let right_block = self.builder.append_basic_block("or_right")?;
                    let end_block = self.builder.append_basic_block("or_end")?;

                    self.builder.position_at_end(&pre_block);
                    self.builder.unconditional_branch(&left_block)?;

                    self.builder.position_at_end(&left_block);
                    let left_value = self.visit_expr(left)?;
                    self.builder
                        .conditional_branch(&left_value, &end_block, &right_block)?;

                    self.builder.position_at_end(&right_block);
                    let right_value = self.visit_expr(right)?;
                    self.builder.unconditional_branch(&end_block)?;

                    self.builder.position_at_end(&end_block);
                    result = self.builder.phi(
                        r#type,
                        &[(left_value, left_block), (right_value, right_block)],
                    )?;
                }
                BinOpKind::Assign => {
                    let right_value = self.visit_expr(right)?;
                    self.build_store(left, &right_value)?;
                    result = right_value;
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
                    let right_value = self.visit_expr(&eq_expr)?;
                    self.build_store(left, &right_value)?;
                    result = right_value;
                }
                _ => {
                    let left_value = self.visit_expr(left)?;
                    let left_type = &left.borrow().r#type;
                    let right_value = self.visit_expr(right)?;
                    let right_type = &right.borrow().r#type;
                    result = self.builder.binop(
                        *op,
                        &left_value,
                        left_type,
                        &right_value,
                        right_type,
                    )?;
                }
            },
            _ => unreachable!(),
        }

        self.builder.pop_context();

        Ok(result)
    }

    //如果expr是位域访问, 那么生成加载位域值的操作, 否则就是正常的load代码
    pub fn build_load(&mut self, node: &Rc<RefCell<Expr>>) -> Result<B::Value, Diagnostic<usize>> {
        let value = self.visit_expr(node)?;
        let value = self.builder.load(&value, &node.borrow().r#type)?;

        match &*node.borrow() {
            Expr {
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
                        if let Some(ConstDesignation::MemberAccess(t)) = &child.designation
                            && *t == *name
                        {
                            return self
                                .builder
                                .load_bitfield(&value, child.width, child.offset);
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
        node: &Rc<RefCell<Expr>>,
        value: &B::Value,
    ) -> Result<(), Diagnostic<usize>> {
        let ptr = self.visit_expr(node)?;

        match &*node.borrow() {
            Expr {
                kind:
                    ExprKind::MemberAccess {
                        target,
                        is_arrow,
                        name,
                    },
                symbol: Some(symbol),
                r#type,
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
                        if let Some(ConstDesignation::MemberAccess(t)) = &child.designation
                            && *t == *name
                        {
                            self.builder.store_bitfield(
                                &ptr,
                                r#type,
                                &value,
                                child.width,
                                child.offset,
                            )?;
                            break;
                        }
                    }
                }
            }
            _ => self.builder.store(&ptr, value)?,
        }

        Ok(())
    }
}
