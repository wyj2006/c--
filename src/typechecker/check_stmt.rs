use codespan_reporting::diagnostic::{Diagnostic, Label, LabelStyle};

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
    symtab::{Namespace, Symbol, SymbolKind},
    typechecker::Context,
    variant::Variant,
};
use std::{cell::RefCell, rc::Rc};

impl TypeChecker {
    pub fn visit_stmt(&mut self, node: Rc<RefCell<Stmt>>) -> Result<(), Diagnostic<usize>> {
        self.contexts
            .push(Context::Stmt(node.borrow().kind.clone()));

        let mut errs = Vec::new(); //消歧义时产生的错误
        //消歧义
        let mut new_expr = None;
        let mut new_decls = None;
        match &mut *node.borrow_mut() {
            Stmt {
                kind:
                    StmtKind::DeclExpr {
                        decls: Some(decls),
                        expr: Some(expr),
                        ..
                    },
                ..
            } => {
                //消歧义
                match self.visit_expr(Rc::clone(expr)) {
                    Ok(_) => new_expr = Some(Rc::clone(expr)),
                    Err(e) => {
                        errs.push(e);
                    }
                }
                'outer: loop {
                    for decl in decls.iter() {
                        match self.visit_declaration(Rc::clone(decl)) {
                            Ok(_) => {}
                            Err(e) => {
                                errs.push(e);
                                break 'outer;
                            }
                        }
                    }
                    new_decls = Some(decls.clone());
                    break;
                }
            }
            _ => {}
        }
        match &mut node.borrow_mut().kind {
            StmtKind::DeclExpr {
                expr: expr @ Some(_),
                decls: decls @ Some(_),
            } => {
                *expr = new_expr;
                *decls = new_decls;

                match (decls, expr) {
                    //在消歧义的时候就已经完成了检查
                    (Some(_), None) | (None, Some(_)) => {
                        self.contexts.pop();
                        return Ok(());
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        //给复合语句使用
        let rc_node = Rc::clone(&node);

        let mut node = node.borrow_mut();
        match &node.kind {
            StmtKind::Compound(stmts) => {
                let need_new_scope = !self.contexts.iter().any(|context| match context {
                    //属于函数定义的复合语句不用单独创建作用域
                    Context::Decl(DeclarationKind::Function {
                        body: Some(body), ..
                    }) => Rc::ptr_eq(&rc_node, body),
                    _ => false,
                });
                if need_new_scope {
                    self.enter_scope();
                }

                for stmt in stmts {
                    self.visit_stmt(Rc::clone(stmt))?;
                }

                if need_new_scope {
                    node.symtab = Some(self.leave_scope());
                }
            }
            StmtKind::DeclExpr {
                decls: Some(_),
                expr: Some(_),
            } => {
                return Err(Diagnostic::error()
                    .with_message(format!("cannot disambiguate declaration and expression"))
                    .with_label(Label::primary(node.file_id, node.span)));
            }
            StmtKind::DeclExpr {
                decls: Some(decls),
                expr: None,
            } => {
                for decl in decls {
                    self.visit_declaration(Rc::clone(decl))?;
                }
            }
            StmtKind::DeclExpr {
                decls: None,
                expr: Some(expr),
            } => {
                self.visit_expr(Rc::clone(expr))?;
            }
            StmtKind::DeclExpr {
                decls: None,
                expr: None,
            } => {
                return Err(Diagnostic::error()
                    .with_message(format!("errors occurred during disambiguation"))
                    .with_label(
                        Label::primary(node.file_id, node.span)
                            .with_message("the location where ambiguity occurs"),
                    )
                    .with_labels({
                        let mut labels = Vec::new();

                        for err in errs {
                            for label in err.labels {
                                if let LabelStyle::Primary = label.style {
                                    labels.push(
                                        Label::primary(label.file_id, label.range)
                                            .with_message(err.message.clone()),
                                    );
                                    break;
                                }
                            }
                        }

                        labels
                    }));
            }
            StmtKind::Goto(name) => {
                if let None = self
                    .func_symtabs
                    .last()
                    .unwrap()
                    .borrow()
                    .lookup(Namespace::Label, name)
                {
                    return Err(Diagnostic::error()
                        .with_message(format!("label '{name}' is undefined"))
                        .with_label(Label::primary(node.file_id, node.span)));
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
                    return Err(Diagnostic::error()
                        .with_message(format!("if condition must have a scale type"))
                        .with_label(Label::primary(
                            condition.borrow().file_id,
                            condition.borrow().span,
                        )));
                }

                self.visit_stmt(Rc::clone(body))?;
                if let Some(t) = else_body {
                    self.visit_stmt(Rc::clone(t))?;
                }

                node.symtab = Some(self.leave_scope());
            }
            StmtKind::Label { name, stmt } => {
                match self.func_symtabs.last_mut() {
                    Some(t) => t.borrow_mut().add(
                        Namespace::Label,
                        Rc::new(RefCell::new(Symbol {
                            define_loc: Some((node.file_id, node.span)),
                            declare_locs: vec![(node.file_id, node.span)],
                            name: name.clone(),
                            kind: SymbolKind::Label,
                            r#type: Rc::new(RefCell::new(Type {
                                kind: TypeKind::Void,
                                ..Type::new(node.file_id, node.span)
                            })),
                            attributes: node.attributes.clone(),
                        })),
                    )?,
                    None => {
                        return Err(Diagnostic::error()
                            .with_message(format!("label must in a function"))
                            .with_label(Label::primary(node.file_id, node.span)));
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
                    return Err(Diagnostic::error()
                        .with_message(format!("return statement must in a function"))
                        .with_label(Label::primary(node.file_id, node.span)));
                }
                if let Some(t) = expr {
                    self.visit_expr(Rc::clone(t))?;
                }
                match &mut node.kind {
                    StmtKind::Return { expr: Some(expr) } => {
                        let TypeKind::Function { return_type, .. } =
                            &self.func_types.last().unwrap().borrow().kind
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
                    return Err(Diagnostic::error()
                        .with_message(format!("switch condition must be an integer"))
                        .with_label(Label::primary(
                            condition.borrow().file_id,
                            condition.borrow().span,
                        )));
                }

                self.visit_stmt(Rc::clone(body))?;

                node.symtab = Some(self.leave_scope());
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
                    return Err(Diagnostic::error()
                        .with_message(format!("case statement must in a switch statement"))
                        .with_label(Label::primary(node.file_id, node.span)));
                }

                self.visit_expr(Rc::clone(expr))?;
                match &expr.borrow().value {
                    Variant::Int(_) => {}
                    _ => {
                        return Err(Diagnostic::error().with_message(
                            "expression in 'case' must be an integer constant expression",
                        ));
                    }
                }
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
                    return Err(Diagnostic::error()
                        .with_message(format!("default statement must in a switch statement"))
                        .with_label(Label::primary(node.file_id, node.span)));
                }
                if let Some(t) = stmt {
                    self.visit_stmt(Rc::clone(t))?;
                }
            }
            StmtKind::While { condition, body } => {
                self.enter_scope();

                self.visit_expr(Rc::clone(condition))?;
                if !condition.borrow().r#type.borrow().is_scale() {
                    return Err(Diagnostic::error()
                        .with_message(format!("switch condition must have a scale type"))
                        .with_label(Label::primary(
                            condition.borrow().file_id,
                            condition.borrow().span,
                        )));
                }

                self.visit_stmt(Rc::clone(body))?;

                node.symtab = Some(self.leave_scope());
            }
            StmtKind::DoWhile { condition, body } => {
                self.enter_scope();

                self.visit_expr(Rc::clone(condition))?;
                if !condition.borrow().r#type.borrow().is_scale() {
                    return Err(Diagnostic::error()
                        .with_message(format!("do-while condition must have a scale type"))
                        .with_label(Label::primary(
                            condition.borrow().file_id,
                            condition.borrow().span,
                        )));
                }

                self.visit_stmt(Rc::clone(body))?;

                node.symtab = Some(self.leave_scope());
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
                    return Err(Diagnostic::error()
                        .with_message(format!(
                            "break statement must in a loop or a switch statement"
                        ))
                        .with_label(Label::primary(node.file_id, node.span)));
                }
            }
            StmtKind::Continue => {
                if !self.contexts.iter().any(|context| match context {
                    Context::Stmt(
                        StmtKind::While { .. } | StmtKind::DoWhile { .. } | StmtKind::For { .. },
                    ) => true,
                    _ => false,
                }) {
                    return Err(Diagnostic::error()
                        .with_message(format!("continue statement must in a loop"))
                        .with_label(Label::primary(node.file_id, node.span)));
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
                        return Err(Diagnostic::error()
                            .with_message(format!("for condition must have a scale type"))
                            .with_label(Label::primary(t.borrow().file_id, t.borrow().span)));
                    }
                }

                if let Some(t) = iter_expr {
                    self.visit_expr(Rc::clone(t))?;
                }

                self.visit_stmt(Rc::clone(body))?;

                node.symtab = Some(self.leave_scope());
            }
        }

        self.contexts.pop();
        Ok(())
    }
}
