use crate::{
    ast::stmt::{Stmt, StmtKind},
    codegen::riscv::{
        A0_REG, A1_REG, CodeGen, FA0_REG,
        instruction::{Opcode, Operand},
    },
    ctype::get_inner_type,
    optimizer::constfolder::ConstFolder,
    symtab::Namespace,
};
use codespan_reporting::diagnostic::Diagnostic;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

impl CodeGen {
    pub fn visit_stmt(&mut self, node: &Rc<RefCell<Stmt>>) -> Result<(), Diagnostic<usize>> {
        let node_key = node.as_ptr() as usize;

        let Stmt { symtab, kind, .. } = &*node.borrow();

        if let Some(t) = symtab {
            self.enter_scope(t.clone());
        }

        match kind {
            StmtKind::Compound(stmts) => {
                for stmt in stmts {
                    self.visit_stmt(stmt)?;
                }
            }
            StmtKind::DeclExpr { decls, expr } => {
                if let Some(decls) = decls {
                    for decl in decls {
                        self.visit_declaration(decl)?;
                    }
                }
                if let Some(expr) = expr {
                    self.visit_expr(expr)?;
                }
            }
            StmtKind::Null => {}
            StmtKind::Break(Some(stmt)) => {
                let key = stmt.as_ptr() as usize;
                let label = self.stmt_labels.get(&key).unwrap().get(0).unwrap().clone();
                self.add_instruction(Opcode::Jump, &[label])?;
            }
            StmtKind::Continue(Some(stmt)) => {
                let key = stmt.as_ptr() as usize;
                let label = self.stmt_labels.get(&key).unwrap().get(1).unwrap().clone();
                self.add_instruction(Opcode::Jump, &[label])?;
            }
            StmtKind::Goto(name) => {
                let symbol = self.lookup(Namespace::Label, name).unwrap();
                let key = symbol.as_ptr() as usize;
                let label = self.stmt_labels.get(&key).unwrap().get(0).unwrap().clone();
                self.add_instruction(Opcode::Jump, &[label])?;
            }
            StmtKind::Label { name, stmt } => {
                let symbol = self.lookup(Namespace::Label, name).unwrap();
                let key = symbol.as_ptr() as usize;
                self.stmt_labels
                    .insert(key, vec![Operand::Symbol(name.to_string())]);

                if let Some(stmt) = stmt {
                    self.visit_stmt(stmt)?;
                }
            }
            StmtKind::If {
                condition,
                body,
                else_body,
            } => {
                let cond_block = self.current_basic_block();
                let then_block = self.append_basic_block("if_then")?;
                self.position_at_end(&then_block)?;
                let else_block = self.append_basic_block("if_else")?;
                self.position_at_end(&else_block)?;
                let merge_block = self.append_basic_block("if_merge")?;

                self.position_at_end(&cond_block)?;
                let cond = self.visit_expr(condition)?;
                let cond = self.to_bool(&cond, Some(&condition.borrow().r#type))?;
                self.add_instruction(Opcode::BEqZ, &[cond, Operand::Symbol(else_block.clone())])?;

                self.position_at_end(&then_block)?;
                self.visit_stmt(body)?;
                self.add_instruction(Opcode::Jump, &[Operand::Symbol(merge_block.clone())])?;

                self.position_at_end(&else_block)?;
                if let Some(else_body) = else_body {
                    self.visit_stmt(else_body)?;
                }

                self.position_at_end(&merge_block)?;
            }
            StmtKind::While { condition, body } => {
                let cond_block = self.append_basic_block("while_cond")?;
                self.position_at_end(&cond_block)?;
                let body_block = self.append_basic_block("while_body")?;
                self.position_at_end(&body_block)?;
                let exit_block = self.append_basic_block("while_exit")?;

                self.stmt_labels.insert(
                    node_key,
                    vec![
                        Operand::Symbol(exit_block.clone()),
                        Operand::Symbol(cond_block.clone()),
                    ],
                );

                self.position_at_end(&cond_block)?;
                let cond = self.visit_expr(condition)?;
                let cond = self.to_bool(&cond, Some(&condition.borrow().r#type))?;
                self.add_instruction(Opcode::BEqZ, &[cond, Operand::Symbol(exit_block.clone())])?;

                self.position_at_end(&body_block)?;
                self.visit_stmt(body)?;
                self.add_instruction(Opcode::Jump, &[Operand::Symbol(cond_block.clone())])?;

                self.position_at_end(&exit_block)?;
            }
            StmtKind::DoWhile { condition, body } => {
                let body_block = self.append_basic_block("dowhile_body")?;
                self.position_at_end(&body_block)?;
                let cond_block = self.append_basic_block("dowhile_cond")?;
                self.position_at_end(&cond_block)?;
                let exit_block = self.append_basic_block("dowhile_exit")?;

                self.stmt_labels.insert(
                    node_key,
                    vec![
                        Operand::Symbol(exit_block.clone()),
                        Operand::Symbol(cond_block.clone()),
                    ],
                );

                self.position_at_end(&body_block)?;
                self.visit_stmt(body)?;
                self.add_instruction(Opcode::Jump, &[Operand::Symbol(cond_block.clone())])?;

                self.position_at_end(&cond_block)?;
                let cond = self.visit_expr(condition)?;
                let cond = self.to_bool(&cond, Some(&condition.borrow().r#type))?;
                self.add_instruction(Opcode::BEqZ, &[cond, Operand::Symbol(exit_block.clone())])?;
                self.add_instruction(Opcode::Jump, &[Operand::Symbol(body_block.clone())])?;

                self.position_at_end(&exit_block)?;
            }
            StmtKind::For {
                init_expr,
                init_decl,
                condition,
                iter_expr,
                body,
            } => {
                if let Some(expr) = init_expr {
                    self.visit_expr(expr)?;
                }
                if let Some(decl) = init_decl {
                    self.visit_declaration(decl)?;
                }

                let cond_block = self.append_basic_block("for_cond")?;
                self.position_at_end(&cond_block)?;
                let body_block = self.append_basic_block("for_body")?;
                self.position_at_end(&body_block)?;
                let iter_block = self.append_basic_block("for_iter")?;
                self.position_at_end(&iter_block)?;
                let exit_block = self.append_basic_block("for_exit")?;

                self.stmt_labels.insert(
                    node_key,
                    vec![
                        Operand::Symbol(exit_block.clone()),
                        Operand::Symbol(iter_block.clone()),
                    ],
                );

                self.position_at_end(&cond_block)?;
                if let Some(condition) = condition {
                    let cond = self.visit_expr(condition)?;
                    let cond = self.to_bool(&cond, Some(&condition.borrow().r#type))?;
                    self.add_instruction(
                        Opcode::BEqZ,
                        &[cond, Operand::Symbol(exit_block.clone())],
                    )?;
                }
                self.add_instruction(Opcode::Jump, &[Operand::Symbol(body_block.clone())])?;

                self.position_at_end(&body_block)?;
                self.visit_stmt(body)?;
                self.add_instruction(Opcode::Jump, &[Operand::Symbol(iter_block.clone())])?;

                self.position_at_end(&iter_block)?;
                if let Some(iter_expr) = iter_expr {
                    self.visit_expr(iter_expr)?;
                }
                self.add_instruction(Opcode::Jump, &[Operand::Symbol(cond_block.clone())])?;

                self.position_at_end(&exit_block)?;
            }
            StmtKind::Case { expr: _, stmt } => {
                //expr是常量表达式, 交给switch处理
                let block = self.append_basic_block("case")?;
                self.position_at_end(&block)?;

                self.stmt_labels
                    .insert(node_key, vec![Operand::Symbol(block.clone())]);

                if let Some(stmt) = stmt {
                    self.visit_stmt(stmt)?;
                }
            }
            StmtKind::Default(stmt) => {
                let block = self.append_basic_block("default")?;
                self.position_at_end(&block)?;

                self.stmt_labels
                    .insert(node_key, vec![Operand::Symbol(block.clone())]);

                if let Some(stmt) = stmt {
                    self.visit_stmt(stmt)?;
                }
            }
            StmtKind::Switch {
                condition,
                body,
                cases_or_default,
            } => {
                let cond_block = self.current_basic_block();
                let exit_block = self.append_basic_block("switch_exit")?;

                self.stmt_labels
                    .insert(node_key, vec![Operand::Symbol(exit_block.clone())]);

                let cond = self.visit_expr(condition)?;

                self.visit_stmt(body)?;

                self.position_at_end(&cond_block)?;

                let mut default_block = None;
                for case_or_default in cases_or_default {
                    let key = case_or_default.as_ptr() as usize;
                    match &case_or_default.borrow().kind {
                        StmtKind::Case { expr, .. } => {
                            ConstFolder::new().visit_expr(expr.clone(), HashMap::new())?;
                            let value = self
                                .variant_to_operand(&expr.borrow().value, &expr.borrow().r#type)?;
                            self.add_instruction(
                                Opcode::BEq,
                                &[
                                    cond.clone(),
                                    value.clone(),
                                    self.stmt_labels.get(&key).unwrap()[0].clone(),
                                ],
                            )?;
                        }
                        StmtKind::Default(_) => {
                            default_block = Some(self.stmt_labels.get(&key).unwrap()[0].clone());
                        }
                        _ => unreachable!(),
                    }
                }
                if let Some(t) = default_block {
                    self.add_instruction(Opcode::Jump, &[t])?;
                }

                self.position_at_end(&exit_block)?;
            }
            StmtKind::Return { expr } => {
                if let Some(expr) = expr {
                    let xsize = self.xlen / 8;
                    let ret_value = self.visit_expr(expr)?;
                    match &get_inner_type(expr.borrow().r#type.clone()).borrow().kind {
                        t if t.is_float_type() => {
                            self.add_instruction(Opcode::FMoveS, &[FA0_REG, ret_value])?;
                        }
                        t if t.is_double() => {
                            self.add_instruction(Opcode::FMoveD, &[FA0_REG, ret_value])?;
                        }
                        t if t.is_scale() => {
                            self.add_instruction(Opcode::Move, &[A0_REG, ret_value])?;
                        }
                        //这时的ret_value应该是一个指针
                        t if t.is_aggregate() => {
                            let size = t.size().unwrap();
                            if size > xsize * 2 {
                                self.call_memcpy(
                                    &A0_REG,
                                    &ret_value,
                                    &Operand::Immediate(size as i64),
                                )?;
                            } else {
                                let load_opcode = match self.xlen {
                                    32 => Opcode::LoadWU,
                                    64 => Opcode::LoadD,
                                    _ => unreachable!(),
                                };
                                self.add_instruction(load_opcode, &[A0_REG, ret_value.clone()])?;

                                if size > xsize {
                                    self.add_instruction(
                                        load_opcode,
                                        &[
                                            A1_REG,
                                            Operand::Address {
                                                base: Box::new(ret_value),
                                                offset: xsize as i64,
                                            },
                                        ],
                                    )?;
                                }
                            }
                        }
                        _ => unreachable!(),
                    }
                }

                self.restore_callee_regs()?;

                self.add_instruction(Opcode::Ret, &[])?;
            }
            _ => {}
        }

        if let Some(_) = symtab {
            self.leave_scope();
        }

        Ok(())
    }
}
