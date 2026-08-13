use crate::{
    ast::stmt::{Stmt, StmtKind},
    legalizer::riscv::Legalizer,
};
use codespan_reporting::diagnostic::Diagnostic;
use std::{cell::RefCell, rc::Rc};

impl Legalizer {
    pub fn visit_stmt(&mut self, node: &Rc<RefCell<Stmt>>) -> Result<(), Diagnostic<usize>> {
        let Stmt { symtab, kind, .. } = &*node.borrow();

        if let Some(symtab) = symtab {
            self.enter_scope(symtab.clone());
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

            StmtKind::DoWhile { condition, body } | StmtKind::While { condition, body } => {
                self.visit_expr(condition)?;
                self.visit_stmt(body)?;
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
                if let Some(condition) = condition {
                    self.visit_expr(condition)?;
                }
                if let Some(iter_expr) = iter_expr {
                    self.visit_expr(iter_expr)?;
                }
                self.visit_stmt(body)?;
            }
            StmtKind::If {
                condition,
                body,
                else_body,
            } => {
                self.visit_expr(condition)?;
                self.visit_stmt(body)?;
                if let Some(else_body) = else_body {
                    self.visit_stmt(else_body)?;
                }
            }
            StmtKind::Switch {
                condition, body, ..
            } => {
                self.visit_expr(condition)?;
                self.visit_stmt(body)?;
            }
            StmtKind::Case { expr, stmt } => {
                self.visit_expr(expr)?;
                if let Some(stmt) = stmt {
                    self.visit_stmt(stmt)?;
                }
            }
            StmtKind::Default(Some(stmt)) => self.visit_stmt(stmt)?,
            StmtKind::Label {
                stmt: Some(stmt), ..
            } => self.visit_stmt(stmt)?,
            StmtKind::Return { expr: Some(expr) } => self.visit_expr(expr)?,
            _ => {}
        }

        if let Some(_) = symtab {
            self.leave_scope();
        }

        Ok(())
    }
}
