use std::{cell::RefCell, rc::Rc};

use codespan_reporting::diagnostic::{Diagnostic, Label};
use inkwell::values::{AnyValueEnum, BasicValue};

use crate::{
    ast::stmt::{Stmt, StmtKind},
    codegen::CodeGen,
    diagnostic::map_builder_err,
    symtab::Namespace,
};

impl<'ctx> CodeGen<'ctx> {
    pub fn visit_stmt(&mut self, node: Rc<RefCell<Stmt>>) -> Result<(), Diagnostic<usize>> {
        let node = node.borrow();
        if let Some(symtab) = &node.symtab {
            self.enter_scope(Rc::clone(symtab));
        }

        match &node.kind {
            StmtKind::Null => {}
            StmtKind::DeclExpr { decls, expr } => {
                if let Some(decls) = decls {
                    for decl in decls {
                        self.visit_declaration(Rc::clone(decl))?;
                    }
                }
                if let Some(expr) = expr {
                    self.visit_expr(Rc::clone(expr))?;
                }
            }
            StmtKind::Compound(stmts) => {
                for stmt in stmts {
                    self.visit_stmt(Rc::clone(stmt))?;
                }
            }
            StmtKind::Break => {
                map_builder_err(
                    node.file_id,
                    node.span,
                    self.builder
                        .build_unconditional_branch(*self.break_blocks.last().unwrap()),
                )?;
                let unreach_block = self
                    .context
                    .append_basic_block(*self.func_values.last().unwrap(), "unreach");
                self.builder.position_at_end(unreach_block);
            }
            StmtKind::Continue => {
                map_builder_err(
                    node.file_id,
                    node.span,
                    self.builder
                        .build_unconditional_branch(*self.continue_blocks.last().unwrap()),
                )?;
                let unreach_block = self
                    .context
                    .append_basic_block(*self.func_values.last().unwrap(), "unreach");
                self.builder.position_at_end(unreach_block);
            }
            StmtKind::Return { expr } => {
                if let Some(expr) = expr {
                    let ret_value = self.visit_expr(Rc::clone(expr))?;
                    let ret_value: Option<&dyn BasicValue<'_>> = match &ret_value {
                        AnyValueEnum::ArrayValue(t) => Some(t),
                        AnyValueEnum::FloatValue(t) => Some(t),
                        AnyValueEnum::IntValue(t) => Some(t),
                        AnyValueEnum::PointerValue(t) => Some(t),
                        AnyValueEnum::ScalableVectorValue(t) => Some(t),
                        AnyValueEnum::StructValue(t) => Some(t),
                        AnyValueEnum::VectorValue(t) => Some(t),
                        _ => {
                            return Err(Diagnostic::error()
                                .with_message("return a invalid value")
                                .with_label(Label::primary(node.file_id, node.span)));
                        }
                    };
                    map_builder_err(
                        node.file_id,
                        node.span,
                        self.builder.build_return(ret_value),
                    )?;
                } else {
                    map_builder_err(node.file_id, node.span, self.builder.build_return(None))?;
                }
                let unreach_block = self
                    .context
                    .append_basic_block(*self.func_values.last().unwrap(), "unreach");
                self.builder.position_at_end(unreach_block);
            }
            StmtKind::If {
                condition,
                body,
                else_body,
            } => {
                let then_block = self
                    .context
                    .append_basic_block(*self.func_values.last().unwrap(), "if_then");
                let else_block = if let Some(_) = else_body {
                    Some(
                        self.context
                            .append_basic_block(*self.func_values.last().unwrap(), "if_else"),
                    )
                } else {
                    None
                };
                let merge_block = self
                    .context
                    .append_basic_block(*self.func_values.last().unwrap(), "if_merge");

                let condition_value = self.to_bool(Rc::clone(condition))?;
                map_builder_err(
                    node.file_id,
                    node.span,
                    self.builder.build_conditional_branch(
                        condition_value,
                        then_block,
                        else_block.unwrap_or(merge_block),
                    ),
                )?;

                then_block
                    .move_after(self.builder.get_insert_block().unwrap())
                    .unwrap();

                self.builder.position_at_end(then_block);
                self.visit_stmt(Rc::clone(body))?;
                map_builder_err(
                    node.file_id,
                    node.span,
                    self.builder.build_unconditional_branch(merge_block),
                )?;

                if let Some(else_block) = else_block {
                    else_block
                        .move_after(self.builder.get_insert_block().unwrap())
                        .unwrap();

                    self.builder.position_at_end(else_block);
                    if let Some(else_body) = else_body {
                        self.visit_stmt(Rc::clone(else_body))?;
                    }
                    map_builder_err(
                        node.file_id,
                        node.span,
                        self.builder.build_unconditional_branch(merge_block),
                    )?;
                }

                merge_block
                    .move_after(self.builder.get_insert_block().unwrap())
                    .unwrap();

                self.builder.position_at_end(merge_block);
            }
            StmtKind::While { condition, body } => {
                let cond_block = self
                    .context
                    .append_basic_block(*self.func_values.last().unwrap(), "while_cond");
                let body_block = self
                    .context
                    .append_basic_block(*self.func_values.last().unwrap(), "while_body");
                let exit_block = self
                    .context
                    .append_basic_block(*self.func_values.last().unwrap(), "while_exit");

                map_builder_err(
                    node.file_id,
                    node.span,
                    self.builder.build_unconditional_branch(cond_block),
                )?;

                self.builder.position_at_end(cond_block);
                let condition_value = self.to_bool(Rc::clone(condition))?;
                map_builder_err(
                    node.file_id,
                    node.span,
                    self.builder
                        .build_conditional_branch(condition_value, body_block, exit_block),
                )?;

                body_block
                    .move_after(self.builder.get_insert_block().unwrap())
                    .unwrap();

                self.builder.position_at_end(body_block);
                self.break_blocks.push(exit_block);
                self.continue_blocks.push(cond_block);
                self.visit_stmt(Rc::clone(body))?;
                self.break_blocks.pop();
                self.continue_blocks.pop();
                map_builder_err(
                    node.file_id,
                    node.span,
                    self.builder.build_unconditional_branch(cond_block),
                )?;

                exit_block
                    .move_after(self.builder.get_insert_block().unwrap())
                    .unwrap();

                self.builder.position_at_end(exit_block);
            }
            StmtKind::DoWhile { condition, body } => {
                let body_block = self
                    .context
                    .append_basic_block(*self.func_values.last().unwrap(), "dowhile_body");
                let cond_block = self
                    .context
                    .append_basic_block(*self.func_values.last().unwrap(), "dowhile_cond");
                let exit_block = self
                    .context
                    .append_basic_block(*self.func_values.last().unwrap(), "dowhile_exit");

                map_builder_err(
                    node.file_id,
                    node.span,
                    self.builder.build_unconditional_branch(body_block),
                )?;

                self.builder.position_at_end(body_block);
                self.break_blocks.push(exit_block);
                self.continue_blocks.push(cond_block);
                self.visit_stmt(Rc::clone(body))?;
                self.break_blocks.pop();
                self.continue_blocks.pop();

                cond_block
                    .move_after(self.builder.get_insert_block().unwrap())
                    .unwrap();

                self.builder.position_at_end(cond_block);
                let condition_value = self.to_bool(Rc::clone(condition))?;
                map_builder_err(
                    node.file_id,
                    node.span,
                    self.builder
                        .build_conditional_branch(condition_value, body_block, exit_block),
                )?;

                exit_block
                    .move_after(self.builder.get_insert_block().unwrap())
                    .unwrap();

                self.builder.position_at_end(exit_block);
            }
            StmtKind::For {
                init_expr,
                init_decl,
                condition,
                iter_expr,
                body,
            } => {
                if let Some(init_expr) = init_expr {
                    self.visit_expr(Rc::clone(init_expr))?;
                }
                if let Some(init_decl) = init_decl {
                    self.visit_declaration(Rc::clone(init_decl))?;
                }

                let cond_block = self
                    .context
                    .append_basic_block(*self.func_values.last().unwrap(), "for_cond");
                let body_block = self
                    .context
                    .append_basic_block(*self.func_values.last().unwrap(), "for_body");
                let iter_block = self
                    .context
                    .append_basic_block(*self.func_values.last().unwrap(), "for_inc");
                let exit_block = self
                    .context
                    .append_basic_block(*self.func_values.last().unwrap(), "for_exit");

                map_builder_err(
                    node.file_id,
                    node.span,
                    self.builder.build_unconditional_branch(cond_block),
                )?;

                self.builder.position_at_end(cond_block);
                if let Some(condition) = condition {
                    let condition_value = self.to_bool(Rc::clone(condition))?;
                    map_builder_err(
                        node.file_id,
                        node.span,
                        self.builder.build_conditional_branch(
                            condition_value,
                            body_block,
                            exit_block,
                        ),
                    )?;
                }

                body_block
                    .move_after(self.builder.get_insert_block().unwrap())
                    .unwrap();

                self.builder.position_at_end(body_block);
                self.break_blocks.push(exit_block);
                self.continue_blocks.push(iter_block);
                self.visit_stmt(Rc::clone(body))?;
                self.break_blocks.pop();
                self.continue_blocks.pop();
                map_builder_err(
                    node.file_id,
                    node.span,
                    self.builder.build_unconditional_branch(iter_block),
                )?;

                iter_block
                    .move_after(self.builder.get_insert_block().unwrap())
                    .unwrap();

                self.builder.position_at_end(iter_block);
                if let Some(iter_expr) = iter_expr {
                    self.visit_expr(Rc::clone(iter_expr))?;
                }
                map_builder_err(
                    node.file_id,
                    node.span,
                    self.builder.build_unconditional_branch(cond_block),
                )?;

                exit_block
                    .move_after(self.builder.get_insert_block().unwrap())
                    .unwrap();

                self.builder.position_at_end(exit_block);
            }
            StmtKind::Label { name, stmt } => {
                let label_block = self
                    .context
                    .append_basic_block(*self.func_values.last().unwrap(), &name);
                self.label_blocks.insert(
                    self.lookup(Namespace::Label, name).unwrap().as_ptr() as usize,
                    label_block,
                );

                self.builder.position_at_end(label_block);
                if let Some(stmt) = stmt {
                    self.visit_stmt(Rc::clone(stmt))?;
                }
            }
            StmtKind::Goto(name) => {
                map_builder_err(
                    node.file_id,
                    node.span,
                    self.builder.build_unconditional_branch(
                        *self
                            .label_blocks
                            .get(&(self.lookup(Namespace::Label, name).unwrap().as_ptr() as usize))
                            .unwrap(),
                    ),
                )?;
            }
            StmtKind::Switch { condition, body } => {
                let cond_block = self.builder.get_insert_block().unwrap();
                let condition_value = self.visit_expr(Rc::clone(condition))?;

                let exit_block = self
                    .context
                    .append_basic_block(*self.func_values.last().unwrap(), "switch_exit");

                self.cases_or_default.push(vec![]);
                self.break_blocks.push(exit_block);
                self.visit_stmt(Rc::clone(&body))?;
                self.break_blocks.pop();

                exit_block
                    .move_after(self.builder.get_insert_block().unwrap())
                    .unwrap();

                let mut default_block = None;
                let mut cases = Vec::new();
                for i in self.cases_or_default.pop().unwrap() {
                    if let (Some(a), b) = i {
                        cases.push((a, b));
                    } else {
                        default_block = Some(i.1);
                    }
                }

                self.builder.position_at_end(cond_block);
                map_builder_err(
                    node.file_id,
                    node.span,
                    self.builder.build_switch(
                        condition_value.into_int_value(),
                        default_block.unwrap_or(exit_block),
                        &cases,
                    ),
                )?;

                self.builder.position_at_end(exit_block);
            }
            StmtKind::Case { expr, stmt } => {
                let case_block = self
                    .context
                    .append_basic_block(*self.func_values.last().unwrap(), "switch_case");

                self.builder.position_at_end(case_block);
                if let Some(stmt) = stmt {
                    self.visit_stmt(Rc::clone(stmt))?;
                }
                //expr是常量表达式
                let case_value = self.to_llvm_value(
                    expr.borrow().value.clone(),
                    Rc::clone(&expr.borrow().r#type),
                )?;
                self.cases_or_default
                    .last_mut()
                    .unwrap()
                    .push((Some(case_value.into_int_value()), case_block));
            }
            StmtKind::Default(stmt) => {
                for i in self.cases_or_default.last().unwrap() {
                    if let None = i.0 {
                        return Err(Diagnostic::error()
                            .with_message("switch only allow on default label")
                            .with_label(Label::primary(node.file_id, node.span)));
                    }
                }
                let default_block = self
                    .context
                    .append_basic_block(*self.func_values.last().unwrap(), "switch_default");

                self.builder.position_at_end(default_block);
                if let Some(stmt) = stmt {
                    self.visit_stmt(Rc::clone(stmt))?;
                }

                self.cases_or_default
                    .last_mut()
                    .unwrap()
                    .push((None, default_block));
            }
        }

        if let Some(_) = node.symtab {
            self.leave_scope();
        }
        Ok(())
    }
}
