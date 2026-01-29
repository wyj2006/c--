use super::TypeChecker;
use crate::{
    ast::stmt::{Stmt, StmtKind},
    ctype::{Type, TypeKind},
    diagnostic::{Error, ErrorKind},
    symtab::{Namespace, Symbol, SymbolKind},
    typechecker::Context,
};
use std::{cell::RefCell, rc::Rc};

impl<'a> TypeChecker<'a> {
    pub fn visit_stmt(&mut self, node: Rc<RefCell<Stmt<'a>>>) -> Result<(), Error<'a>> {
        self.contexts
            .push(Context::Stmt(node.borrow().kind.clone()));

        let node = node.borrow();
        match &node.kind {
            StmtKind::Break => {}
            StmtKind::Case { expr, stmt } => {
                self.visit_expr(Rc::clone(expr))?;
                if let Some(t) = stmt {
                    self.visit_stmt(Rc::clone(t))?;
                }
            }
            StmtKind::Compound(stmts) => {
                for stmt in stmts {
                    self.visit_stmt(Rc::clone(stmt))?;
                }
            }
            StmtKind::Continue => {}
            StmtKind::Decl(decl) => self.visit_declaration(Rc::clone(decl))?,
            StmtKind::Default(stmt) => {
                if let Some(t) = stmt {
                    self.visit_stmt(Rc::clone(t))?;
                }
            }
            StmtKind::DoWhile { condition, body } => {
                self.visit_expr(Rc::clone(condition))?;
                self.visit_stmt(Rc::clone(body))?;
            }
            StmtKind::Expr(expr) => self.visit_expr(Rc::clone(expr))?,
            StmtKind::For {
                init_expr,
                init_decl,
                condition,
                iter_expr,
                body,
            } => {
                if let Some(t) = init_expr {
                    self.visit_expr(Rc::clone(t))?;
                }
                if let Some(t) = init_decl {
                    self.visit_declaration(Rc::clone(t))?;
                }
                if let Some(t) = condition {
                    self.visit_expr(Rc::clone(t))?;
                }
                if let Some(t) = iter_expr {
                    self.visit_expr(Rc::clone(t))?;
                }
                self.visit_stmt(Rc::clone(body))?;
            }
            StmtKind::Goto(name) => {
                if let None = self
                    .func_symtab
                    .last()
                    .unwrap()
                    .borrow()
                    .lookup(Namespace::Label, name)
                {
                    return Err(Error {
                        span: node.span,
                        kind: ErrorKind::Undefine(name.clone()),
                    });
                }
            }
            StmtKind::If {
                condition,
                body,
                else_body,
            } => {
                self.visit_expr(Rc::clone(condition))?;
                self.visit_stmt(Rc::clone(body))?;
                if let Some(t) = else_body {
                    self.visit_stmt(Rc::clone(t))?;
                }
            }
            StmtKind::Label { name, stmt } => {
                self.func_symtab.last_mut().unwrap().borrow_mut().add(
                    Namespace::Label,
                    Rc::new(RefCell::new(Symbol {
                        define_span: Some(node.span),
                        declare_spans: vec![node.span],
                        name: name.clone(),
                        kind: SymbolKind::Label,
                        r#type: Rc::new(RefCell::new(Type {
                            span: node.span,
                            attributes: vec![],
                            kind: TypeKind::Void,
                        })),
                        attributes: node.attributes.clone(),
                    })),
                )?;
                if let Some(t) = stmt {
                    self.visit_stmt(Rc::clone(t))?;
                }
            }
            StmtKind::Null => {}
            StmtKind::Return { expr } => {
                if let Some(t) = expr {
                    self.visit_expr(Rc::clone(t))?;
                }
            }
            StmtKind::Switch { condition, body } => {
                self.visit_expr(Rc::clone(condition))?;
                self.visit_stmt(Rc::clone(body))?;
            }
            StmtKind::While { condition, body } => {
                self.visit_expr(Rc::clone(condition))?;
                self.visit_stmt(Rc::clone(body))?;
            }
        }

        self.contexts.pop();
        Ok(())
    }
}
