use super::TypeChecker;
use crate::{
    ast::{
        decl::DeclarationKind,
        stmt::{Stmt, StmtKind},
    },
    ctype::{
        Type, TypeKind,
        cast::{integer_promote, remove_qualifier},
    },
    diagnostic::{Diagnostic, DiagnosticKind},
    symtab::{Namespace, Symbol, SymbolKind},
    typechecker::Context,
};
use std::{cell::RefCell, rc::Rc};

impl<'a> TypeChecker<'a> {
    pub fn visit_stmt(&mut self, node: Rc<RefCell<Stmt<'a>>>) -> Result<(), Diagnostic<'a>> {
        self.contexts
            .push(Context::Stmt(node.borrow().kind.clone()));

        //给复合语句使用
        let rc_node = Rc::clone(&node);

        let mut node = node.borrow_mut();
        match &node.kind {
            StmtKind::Compound(stmts) => {
                if self.contexts.iter().any(|context| match context {
                    Context::Decl(DeclarationKind::Function {
                        body: Some(body), ..
                    }) => Rc::ptr_eq(&rc_node, body),
                    _ => false,
                }) {
                    //属于函数定义的复合语句不用单独创建作用域
                    for stmt in stmts {
                        self.visit_stmt(Rc::clone(stmt))?;
                    }
                } else {
                    self.enter_scope();
                    for stmt in stmts {
                        self.visit_stmt(Rc::clone(stmt))?;
                    }
                    self.leave_scope();
                }
            }
            StmtKind::Decl(decls) => {
                for decl in decls {
                    self.visit_declaration(Rc::clone(decl))?;
                }
            }
            StmtKind::Expr(expr) => self.visit_expr(Rc::clone(expr))?,
            StmtKind::Goto(name) => {
                if let None = self
                    .func_symtabs
                    .last()
                    .unwrap()
                    .borrow()
                    .lookup(Namespace::Label, name)
                {
                    return Err(Diagnostic {
                        span: node.span,
                        kind: DiagnosticKind::Error,
                        message: format!("label '{name}' is undefined"),
                        notes: vec![],
                    });
                }
            }
            StmtKind::If {
                condition,
                body,
                else_body,
            } => {
                self.enter_scope();
                self.visit_expr(Rc::clone(condition))?;
                if !condition.borrow().r#type.borrow().is_scale() {
                    return Err(Diagnostic {
                        span: condition.borrow().span,
                        kind: DiagnosticKind::Error,
                        message: format!("if condition must have a scale type"),
                        notes: vec![],
                    });
                }
                self.visit_stmt(Rc::clone(body))?;
                if let Some(t) = else_body {
                    self.visit_stmt(Rc::clone(t))?;
                }
                self.leave_scope();
            }
            StmtKind::Label { name, stmt } => {
                match self.func_symtabs.last_mut() {
                    Some(t) => t.borrow_mut().add(
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
                    )?,
                    None => {
                        return Err(Diagnostic {
                            span: node.span,
                            kind: DiagnosticKind::Error,
                            message: format!("'label' must in a function"),
                            notes: vec![],
                        });
                    }
                }

                if let Some(t) = stmt {
                    self.visit_stmt(Rc::clone(t))?;
                }
            }
            StmtKind::Null => {}
            StmtKind::Return { expr } => {
                if !self.contexts.iter().any(|context| match context {
                    Context::Decl(DeclarationKind::Function { .. }) => true,
                    _ => false,
                }) {
                    return Err(Diagnostic {
                        span: node.span,
                        kind: DiagnosticKind::Error,
                        message: format!("'return' statement must in a function"),
                        notes: vec![],
                    });
                }
                if let Some(t) = expr {
                    self.visit_expr(Rc::clone(t))?;
                }
                match &mut node.kind {
                    StmtKind::Return { expr: Some(expr) } => {
                        let TypeKind::Function { return_type, .. } =
                            &self.funcs.last().unwrap().borrow().kind
                        else {
                            unreachable!();
                        };
                        *expr = self.try_implicit_cast(
                            Rc::clone(expr),
                            remove_qualifier(Rc::clone(return_type)),
                        )?;
                    }
                    _ => {}
                }
            }
            StmtKind::Switch { condition, body } => {
                self.enter_scope();
                self.visit_expr(Rc::clone(condition))?;
                if !condition.borrow().r#type.borrow().is_integer() {
                    return Err(Diagnostic {
                        span: condition.borrow().span,
                        kind: DiagnosticKind::Error,
                        message: format!("switch condition must have an integer"),
                        notes: vec![],
                    });
                }
                self.visit_stmt(Rc::clone(body))?;
                self.leave_scope();
            }
            StmtKind::Case { expr, stmt } => {
                let mut condition_type = None;
                for context in &self.contexts {
                    match context {
                        Context::Stmt(StmtKind::Switch { condition, .. }) => {
                            condition_type = Some(Rc::clone(&condition.borrow().r#type));
                            //不break, 以保证获得的condition_type是正确的
                        }
                        _ => {}
                    }
                }

                if let None = condition_type {
                    return Err(Diagnostic {
                        span: node.span,
                        kind: DiagnosticKind::Error,
                        message: format!("'case' statement must in a switch statement"),
                        notes: vec![],
                    });
                }

                self.visit_expr(Rc::clone(expr))?;
                if let Some(t) = stmt {
                    self.visit_stmt(Rc::clone(t))?;
                }

                match &mut node.kind {
                    StmtKind::Case { expr, .. } => {
                        *expr = self.try_implicit_cast(
                            Rc::clone(expr),
                            integer_promote(condition_type.unwrap()),
                        )?;
                    }
                    _ => unreachable!(),
                }
            }
            StmtKind::Default(stmt) => {
                if !self.contexts.iter().any(|context| match context {
                    Context::Stmt(StmtKind::Switch { .. }) => true,
                    _ => false,
                }) {
                    return Err(Diagnostic {
                        span: node.span,
                        kind: DiagnosticKind::Error,
                        message: format!("'default' statement must in a switch statement"),
                        notes: vec![],
                    });
                }
                if let Some(t) = stmt {
                    self.visit_stmt(Rc::clone(t))?;
                }
            }
            StmtKind::While { condition, body } => {
                self.enter_scope();
                self.visit_expr(Rc::clone(condition))?;
                if !condition.borrow().r#type.borrow().is_scale() {
                    return Err(Diagnostic {
                        span: condition.borrow().span,
                        kind: DiagnosticKind::Error,
                        message: format!("while condition must have a scale type"),
                        notes: vec![],
                    });
                }
                self.visit_stmt(Rc::clone(body))?;
                self.leave_scope();
            }
            StmtKind::DoWhile { condition, body } => {
                self.enter_scope();
                self.visit_expr(Rc::clone(condition))?;
                if !condition.borrow().r#type.borrow().is_scale() {
                    return Err(Diagnostic {
                        span: condition.borrow().span,
                        kind: DiagnosticKind::Error,
                        message: format!("do-while condition must have a scale type"),
                        notes: vec![],
                    });
                }
                self.visit_stmt(Rc::clone(body))?;
                self.leave_scope();
            }
            StmtKind::Break => {
                if !self.contexts.iter().any(|context| match context {
                    Context::Stmt(
                        StmtKind::While { .. }
                        | StmtKind::DoWhile { .. }
                        | StmtKind::For { .. }
                        | StmtKind::Switch { .. },
                    ) => true,
                    _ => false,
                }) {
                    return Err(Diagnostic {
                        span: node.span,
                        kind: DiagnosticKind::Error,
                        message: format!("'break' statement must in a loop or a switch statement"),
                        notes: vec![],
                    });
                }
            }
            StmtKind::Continue => {
                if !self.contexts.iter().any(|context| match context {
                    Context::Stmt(
                        StmtKind::While { .. } | StmtKind::DoWhile { .. } | StmtKind::For { .. },
                    ) => true,
                    _ => false,
                }) {
                    return Err(Diagnostic {
                        span: node.span,
                        kind: DiagnosticKind::Error,
                        message: format!("'continue' statement must in a loop"),
                        notes: vec![],
                    });
                }
            }
            StmtKind::For {
                init_expr,
                init_decl,
                condition,
                iter_expr,
                body,
            } => {
                self.enter_scope();
                if let Some(t) = init_expr {
                    self.visit_expr(Rc::clone(t))?;
                }
                if let Some(t) = init_decl {
                    self.visit_declaration(Rc::clone(t))?;
                }
                if let Some(t) = condition {
                    self.visit_expr(Rc::clone(t))?;
                    if !t.borrow().r#type.borrow().is_scale() {
                        return Err(Diagnostic {
                            span: t.borrow().span,
                            kind: DiagnosticKind::Error,
                            message: format!("for condition must have a scale type"),
                            notes: vec![],
                        });
                    }
                }
                if let Some(t) = iter_expr {
                    self.visit_expr(Rc::clone(t))?;
                }
                self.visit_stmt(Rc::clone(body))?;
                self.leave_scope();
            }
        }

        self.contexts.pop();
        Ok(())
    }
}
