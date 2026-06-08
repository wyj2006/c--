use crate::{
    ast::stmt::{Stmt, StmtKind},
    codegen::{CodeGen, builder::Builder},
    symtab::Namespace,
};
use codespan_reporting::diagnostic::{Diagnostic, Label};
use std::{cell::RefCell, rc::Rc};

impl<B: Builder> CodeGen<B> {
    pub fn visit_stmt(&mut self, node: &Rc<RefCell<Stmt>>) -> Result<(), Diagnostic<usize>> {
        let node_ptr = node.as_ptr() as usize;
        let Stmt {
            file_id,
            span,
            attributes: _,
            symtab,
            kind,
        } = &*node.borrow();

        self.builder.append_context(*file_id, *span);

        if let Some(symtab) = symtab {
            self.builder.enter_scope(symtab);
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
            StmtKind::If {
                condition,
                body,
                else_body,
            } => {
                let cond_block = self.builder.current_basic_block()?;
                let then_block = self.builder.append_basic_block("if_then")?;
                let else_block = if let Some(_) = else_body {
                    Some(self.builder.append_basic_block("if_else")?)
                } else {
                    None
                };
                let merge_block = self.builder.append_basic_block("if_merge")?;

                self.builder.position_at_end(&cond_block);
                let cond_value = self.visit_expr(condition)?;
                self.builder.conditional_branch(
                    &cond_value,
                    &then_block,
                    else_block.as_ref().unwrap_or(&merge_block),
                )?;

                self.builder.position_at_end(&then_block);
                self.visit_stmt(body)?;
                self.builder.unconditional_branch(&merge_block)?;

                if let Some(else_body) = else_body {
                    self.builder.position_at_end(else_block.as_ref().unwrap());
                    self.visit_stmt(else_body)?;
                    self.builder.unconditional_branch(&merge_block)?;
                }

                self.builder.position_at_end(&merge_block);
            }
            StmtKind::Switch {
                condition,
                body,
                cases_or_default,
            } => {
                let cond_block = self.builder.current_basic_block()?;
                let exit_block = self.builder.append_basic_block("switch_exit")?;

                self.builder.position_at_end(&cond_block);
                let cond_value = self.visit_expr(condition)?;

                self.visit_stmt(body)?;

                let mut cases = vec![];
                let mut default = None;
                for label in cases_or_default {
                    let ptr = label.as_ptr() as usize;
                    if let Some(t) = self.basic_blocks.get(&ptr) {
                        match &label.borrow().kind {
                            StmtKind::Case { expr, .. } => {
                                let block = t[0].clone();
                                //expr是常量表达式
                                cases.push((self.visit_expr(expr)?, block));
                            }
                            StmtKind::Default { .. } => {
                                if let Some(_) = default {
                                    return Err(Diagnostic::error()
                                        .with_message("switch only allow one default label")
                                        .with_label(Label::primary(*file_id, *span)));
                                } else {
                                    default = Some(t[0].clone())
                                }
                            }
                            _ => unreachable!(),
                        }
                    }
                }

                self.builder.position_at_end(&cond_block);
                self.builder.switch(
                    &cond_value,
                    &condition.borrow().r#type,
                    &cases,
                    default.as_ref().unwrap_or(&exit_block),
                )?;

                self.builder
                    .position_at_end(&self.builder.get_previous_block(&exit_block).unwrap());
                self.builder.unconditional_branch(&exit_block)?;

                self.builder.position_at_end(&exit_block);
            }
            StmtKind::Case { stmt, .. } => {
                let pre_block = self.builder.current_basic_block()?;
                let case_block = self.builder.append_basic_block("switch_case")?;

                self.builder.position_at_end(&pre_block);
                self.builder.unconditional_branch(&case_block)?;

                self.builder.position_at_end(&case_block);
                //expr交给switch生成
                if let Some(stmt) = stmt {
                    self.visit_stmt(stmt)?;
                }

                self.basic_blocks.insert(node_ptr, vec![case_block]);
            }
            StmtKind::Default { stmt, .. } => {
                let pre_block = self.builder.current_basic_block()?;
                let default_block = self.builder.append_basic_block("switch_default")?;

                self.builder.position_at_end(&pre_block);
                self.builder.unconditional_branch(&default_block)?;

                self.builder.position_at_end(&default_block);
                if let Some(stmt) = stmt {
                    self.visit_stmt(stmt)?;
                }

                self.basic_blocks.insert(node_ptr, vec![default_block]);
            }
            StmtKind::While { condition, body } => {
                let pre_block = self.builder.current_basic_block()?;
                let cond_block = self.builder.append_basic_block("while_cond")?;
                let body_block = self.builder.append_basic_block("while_body")?;
                let exit_block = self.builder.append_basic_block("while_exit")?;

                self.builder.position_at_end(&pre_block);
                self.builder.unconditional_branch(&cond_block)?;

                self.basic_blocks
                    .insert(node_ptr, vec![exit_block.clone(), cond_block.clone()]);

                self.builder.position_at_end(&cond_block);
                let cond_value = self.visit_expr(condition)?;
                self.builder
                    .conditional_branch(&cond_value, &body_block, &exit_block)?;

                self.builder.position_at_end(&body_block);
                self.visit_stmt(body)?;
                self.builder.unconditional_branch(&cond_block)?;

                self.builder.position_at_end(&exit_block);
            }
            StmtKind::DoWhile { condition, body } => {
                let pre_block = self.builder.current_basic_block()?;
                let body_block = self.builder.append_basic_block("while_body")?;
                let cond_block = self.builder.append_basic_block("while_cond")?;
                let exit_block = self.builder.append_basic_block("while_exit")?;

                self.builder.position_at_end(&pre_block);
                self.builder.unconditional_branch(&body_block)?;

                self.basic_blocks
                    .insert(node_ptr, vec![exit_block.clone(), cond_block.clone()]);

                self.builder.position_at_end(&body_block);
                self.visit_stmt(body)?;

                self.builder.position_at_end(&cond_block);
                let cond_value = self.visit_expr(condition)?;
                self.builder
                    .conditional_branch(&cond_value, &body_block, &exit_block)?;

                self.builder.position_at_end(&exit_block);
            }
            StmtKind::For {
                init_expr,
                init_decl,
                condition,
                iter_expr,
                body,
            } => {
                if let Some(init_expr) = init_expr {
                    self.visit_expr(init_expr)?;
                }
                if let Some(init_decl) = init_decl {
                    self.visit_declaration(init_decl)?;
                }

                let pre_block = self.builder.current_basic_block()?;
                let cond_block = self.builder.append_basic_block("for_cond")?;
                let body_block = self.builder.append_basic_block("for_body")?;
                let iter_block = self.builder.append_basic_block("for_iter")?;
                let exit_block = self.builder.append_basic_block("for_exit")?;

                self.builder.position_at_end(&pre_block);
                self.builder.unconditional_branch(&cond_block)?;

                self.basic_blocks
                    .insert(node_ptr, vec![exit_block.clone(), iter_block.clone()]);

                self.builder.position_at_end(&cond_block);
                if let Some(condition) = condition {
                    let cond_value = self.visit_expr(condition)?;
                    self.builder
                        .conditional_branch(&cond_value, &body_block, &exit_block)?;
                } else {
                    self.builder.unconditional_branch(&body_block)?;
                }

                self.builder.position_at_end(&body_block);
                self.visit_stmt(body)?;
                self.builder.unconditional_branch(&iter_block)?;

                self.builder.position_at_end(&iter_block);
                if let Some(iter_expr) = iter_expr {
                    self.visit_expr(iter_expr)?;
                }
                self.builder.unconditional_branch(&cond_block)?;

                self.builder.position_at_end(&exit_block);
            }
            StmtKind::Break(Some(loop_stmt)) => {
                self.builder
                    .unconditional_branch(&self.basic_blocks[&(loop_stmt.as_ptr() as usize)][0])?;
                let unreach_block = self.builder.append_basic_block("unreachable")?;
                self.builder.position_at_end(&unreach_block);
            }
            StmtKind::Continue(Some(loop_stmt)) => {
                self.builder
                    .unconditional_branch(&self.basic_blocks[&(loop_stmt.as_ptr() as usize)][1])?;
                let unreach_block = self.builder.append_basic_block("unreachable")?;
                self.builder.position_at_end(&unreach_block);
            }
            StmtKind::Label { name, stmt } => {
                let pre_block = self.builder.current_basic_block()?;
                let label_block = self.builder.append_basic_block(name)?;
                self.basic_blocks.insert(
                    self.builder
                        .lookup(name, Namespace::Label)
                        .unwrap()
                        .as_ptr() as usize,
                    vec![label_block.clone()],
                );

                self.builder.position_at_end(&pre_block);
                self.builder.unconditional_branch(&label_block)?;

                self.builder.position_at_end(&label_block);
                if let Some(stmt) = stmt {
                    self.visit_stmt(stmt)?;
                }
            }
            StmtKind::Goto(name) => {
                self.builder.unconditional_branch(
                    &self.basic_blocks[&(self
                        .builder
                        .lookup(name, Namespace::Label)
                        .unwrap()
                        .as_ptr() as usize)][0],
                )?;

                let next_block = self.builder.append_basic_block("goto_next")?;
                self.builder.position_at_end(&next_block);
            }
            StmtKind::Return { expr } => {
                let value = if let Some(expr) = expr {
                    Some(self.visit_expr(expr)?)
                } else {
                    None
                };
                self.builder.r#return(value)?;

                let unreach_block = self.builder.append_basic_block("unreachable")?;
                self.builder.position_at_end(&unreach_block);
            }
            _ => {}
        }

        if let Some(_) = symtab {
            self.builder.leave_scope();
        }

        self.builder.pop_context();

        Ok(())
    }
}
