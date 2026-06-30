use crate::{
    ast::stmt::{Stmt, StmtKind},
    optimizer::constfolder::ConstFolder,
    symtab::{Namespace, SymbolKind},
    variant::Variant,
};
use codespan_reporting::diagnostic::Diagnostic;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

impl ConstFolder {
    pub fn visit_stmt(
        &mut self,
        node: Rc<RefCell<Stmt>>,
        mut vars: HashMap<String, Variant>,
    ) -> Result<HashMap<String, Variant>, Diagnostic<usize>> {
        let ptr = node.as_ptr() as usize;

        let node = node.borrow();

        match &node.kind {
            StmtKind::Compound(stmts) => {
                for stmt in stmts {
                    vars = self.visit_stmt(Rc::clone(stmt), vars)?;
                }
            }
            StmtKind::If {
                condition,
                body,
                else_body,
            } => {
                vars = self.visit_expr(Rc::clone(condition), vars)?;
                let a = self.visit_stmt(Rc::clone(body), vars.clone())?;

                vars = if let Some(else_body) = else_body {
                    let b = self.visit_stmt(Rc::clone(else_body), vars)?;
                    self.intersection(&a, &b)
                } else {
                    self.intersection(&a, &vars)
                }
            }
            StmtKind::DeclExpr { decls, expr } => {
                if let Some(decls) = decls {
                    for decl in decls {
                        vars = self.visit_declaration(Rc::clone(decl), vars)?;
                    }
                }
                if let Some(expr) = expr {
                    vars = self.visit_expr(Rc::clone(expr), vars)?;
                }
            }
            StmtKind::While { condition, body } => {
                /*     |
                       v
                +> condition -+
                |      |      |
                |      v      |
                ----  body    |
                       |      |
                       v <----+
                */
                let mut condition_in = vars.clone();
                let mut condition_out;
                let mut body_out;

                loop {
                    condition_out = self.visit_expr(Rc::clone(condition), condition_in.clone())?;
                    body_out = self.visit_stmt(Rc::clone(body), condition_out.clone())?;

                    let mut t = self.intersection(&body_out, &vars);
                    if let Some(v) = self.continue_outs.get(&ptr) {
                        for (_, i) in v {
                            t = self.intersection(&t, i);
                        }
                    }

                    if condition_in == t {
                        break;
                    }
                    condition_in = t;
                }

                vars = self.intersection(&condition_out, &body_out);
                if let Some(v) = self.break_outs.get(&ptr) {
                    for (_, i) in v {
                        vars = self.intersection(&vars, i);
                    }
                }
            }
            StmtKind::DoWhile { condition, body } => {
                /*     |
                       v
                +---> body
                |      |
                |      v
                ----condition
                       |
                       v
                */

                let mut body_in = vars.clone();
                let mut body_out;
                let mut condition_in;
                let mut condition_out;

                loop {
                    body_out = self.visit_stmt(Rc::clone(body), body_in.clone())?;

                    condition_in = body_out;
                    if let Some(v) = self.continue_outs.get(&ptr) {
                        for (_, i) in v {
                            condition_in = self.intersection(&condition_in, i);
                        }
                    }
                    condition_out = self.visit_expr(Rc::clone(condition), condition_in)?;

                    let t = self.intersection(&condition_out, &vars);
                    if body_in == t {
                        break;
                    }
                    body_in = t;
                }

                vars = condition_out;
                if let Some(v) = self.break_outs.get(&ptr) {
                    for (_, i) in v {
                        vars = self.intersection(&vars, i);
                    }
                }
            }
            StmtKind::For {
                init_expr,
                init_decl,
                condition,
                iter_expr,
                body,
            } => {
                /*        |
                          v
                init_expr or init_decl
                          |
                          v
                    +>condition--+
                    |     |      |
                    |     v      |
                    |   body     |
                    |     |      |
                    |     v      |
                    - iter_expr  |
                                 |
                          <------+
                */
                if let Some(init_expr) = init_expr {
                    vars = self.visit_expr(Rc::clone(init_expr), vars)?;
                }
                if let Some(init_decl) = init_decl {
                    vars = self.visit_declaration(Rc::clone(&init_decl), vars)?;
                }

                let mut condition_in = vars.clone();
                let mut condition_out;
                let mut body_out;
                let mut iter_expr_in;
                let mut iter_expr_out;

                loop {
                    condition_out = if let Some(condition) = condition {
                        self.visit_expr(Rc::clone(condition), condition_in.clone())?
                    } else {
                        vars.clone()
                    };

                    body_out = self.visit_stmt(Rc::clone(body), condition_out.clone())?;

                    iter_expr_in = body_out;
                    if let Some(v) = self.continue_outs.get(&ptr) {
                        for (_, i) in v {
                            iter_expr_in = self.intersection(&iter_expr_in, i);
                        }
                    }
                    iter_expr_out = if let Some(iter_expr) = iter_expr {
                        self.visit_expr(Rc::clone(iter_expr), iter_expr_in)?
                    } else {
                        iter_expr_in
                    };

                    let t = self.intersection(&iter_expr_out, &vars);
                    if t == condition_in {
                        break;
                    }
                    condition_in = t;
                }

                vars = condition_out;
                if let Some(v) = self.break_outs.get(&ptr) {
                    for (_, i) in v {
                        vars = self.intersection(&vars, i);
                    }
                }
            }
            StmtKind::Break(Some(t)) => {
                match self.break_outs.get_mut(&(t.as_ptr() as usize)) {
                    Some(v) => {
                        v.insert(ptr, vars.clone());
                    }
                    None => {
                        self.break_outs
                            .insert(t.as_ptr() as usize, HashMap::from([(ptr, vars.clone())]));
                    }
                }
                vars = self.unreachable(&vars);
            }
            StmtKind::Continue(Some(t)) => {
                match self.continue_outs.get_mut(&(t.as_ptr() as usize)) {
                    Some(v) => {
                        v.insert(ptr, vars.clone());
                    }
                    None => {
                        self.continue_outs
                            .insert(t.as_ptr() as usize, HashMap::from([(ptr, vars.clone())]));
                    }
                }
                vars = self.unreachable(&vars);
            }
            StmtKind::Return { expr } => {
                if let Some(expr) = expr {
                    vars = self.visit_expr(Rc::clone(expr), vars)?;
                }
                vars = self.unreachable(&vars);
            }
            StmtKind::Label { stmt, .. } => {
                if let Some(v) = self.label_ins.get(&ptr) {
                    for (_, i) in v {
                        vars = self.intersection(&vars, i);
                    }
                }
                if let Some(stmt) = stmt {
                    vars = self.visit_stmt(Rc::clone(stmt), vars)?;
                }
            }
            StmtKind::Case { expr, stmt } => {
                if let Some(v) = self.case_ins.get(&ptr) {
                    for (_, i) in v {
                        vars = self.intersection(&vars, i);
                    }
                }
                vars = self.visit_expr(Rc::clone(expr), vars)?;
                if let Some(stmt) = stmt {
                    vars = self.visit_stmt(Rc::clone(stmt), vars)?;
                }
            }
            StmtKind::Default(stmt) => {
                if let Some(v) = self.case_ins.get(&ptr) {
                    for (_, i) in v {
                        vars = self.intersection(&vars, i);
                    }
                }
                if let Some(stmt) = stmt {
                    vars = self.visit_stmt(Rc::clone(stmt), vars)?;
                }
            }
            StmtKind::Goto(name) => {
                if let Some(t) = self.func_symtabs.last()
                    && let Some(t) = t.borrow().lookup(Namespace::Label, name)
                    && let SymbolKind::Label(Some(t)) = &t.borrow().kind
                {
                    match self.label_ins.get_mut(&(t.as_ptr() as usize)) {
                        Some(v) => {
                            v.insert(ptr, vars.clone());
                        }
                        None => {
                            self.label_ins
                                .insert(t.as_ptr() as usize, HashMap::from([(ptr, vars.clone())]));
                        }
                    }
                }
                vars = self.unreachable(&vars);
            }
            StmtKind::Switch {
                condition,
                body,
                cases_or_default,
            } => {
                vars = self.visit_expr(Rc::clone(condition), vars)?;

                for i in cases_or_default {
                    match self.case_ins.get_mut(&(i.as_ptr() as usize)) {
                        Some(v) => {
                            v.insert(ptr, vars.clone());
                        }
                        None => {
                            self.case_ins
                                .insert(i.as_ptr() as usize, HashMap::from([(ptr, vars.clone())]));
                        }
                    }
                }

                vars = self.visit_stmt(Rc::clone(body), vars)?;

                if let Some(v) = self.break_outs.get(&ptr) {
                    for (_, i) in v {
                        vars = self.intersection(&vars, i);
                    }
                }
            }
            _ => {}
        }

        Ok(vars)
    }
}
