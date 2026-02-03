use super::{CParser, Rule};
use crate::ast::stmt::{Stmt, StmtKind};
use crate::diagnostic::from_pest_span;
use codespan_reporting::diagnostic::Diagnostic;
use pest::iterators::Pair;
use std::cell::RefCell;
use std::rc::Rc;

impl CParser {
    pub fn parse_compound_statement(
        &self,
        rule: Pair<Rule>,
    ) -> Result<Rc<RefCell<Stmt>>, Diagnostic<usize>> {
        let mut stmts = Vec::new();
        let span = rule.as_span();

        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::declaration => {
                    let span = rule.as_span();
                    stmts.push(Rc::new(RefCell::new(Stmt {
                        kind: StmtKind::Decl(self.parse_declaration(rule)?),
                        ..Stmt::new(self.file_id, from_pest_span(span))
                    })));
                }
                Rule::unlabeled_statement => stmts.push(self.parse_unlabeled_statement(rule)?),
                Rule::label => stmts.push(self.parse_label(rule)?),
                _ => unreachable!(),
            }
        }

        Ok(Rc::new(RefCell::new(Stmt {
            kind: StmtKind::Compound(stmts),
            ..Stmt::new(self.file_id, from_pest_span(span))
        })))
    }

    pub fn parse_unlabeled_statement(
        &self,
        rule: Pair<Rule>,
    ) -> Result<Rc<RefCell<Stmt>>, Diagnostic<usize>> {
        let span = rule.as_span();
        let mut attributes = Vec::new();
        let mut stmt = None;
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::attribute_specifier_sequence => {
                    attributes.extend(self.parse_attribute_specifier_sequence(rule)?)
                }
                Rule::compound_statement => stmt = Some(self.parse_compound_statement(rule)?),
                Rule::selection_statement => stmt = Some(self.parse_selection_statement(rule)?),
                Rule::iteration_statement => stmt = Some(self.parse_iteration_statement(rule)?),
                Rule::jump_statement => stmt = Some(self.parse_jump_statement(rule)?),
                Rule::expression => {
                    let span = rule.as_span();
                    stmt = Some(Rc::new(RefCell::new(Stmt {
                        kind: StmtKind::Expr(self.parse_expression(rule)?),
                        ..Stmt::new(self.file_id, from_pest_span(span))
                    })))
                }
                _ => unreachable!(),
            }
        }
        let stmt = stmt.unwrap_or(Rc::new(RefCell::new(Stmt::new(
            self.file_id,
            from_pest_span(span),
        ))));
        stmt.borrow_mut().attributes.extend(attributes);
        Ok(stmt)
    }

    pub fn parse_label(&self, rule: Pair<Rule>) -> Result<Rc<RefCell<Stmt>>, Diagnostic<usize>> {
        let str = rule.as_str();
        let span = rule.as_span();
        let mut attributes = Vec::new();
        let mut name = String::new();
        let mut expr = None;

        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::attribute_specifier_sequence => {
                    attributes.extend(self.parse_attribute_specifier_sequence(rule)?)
                }
                Rule::identifier => name = rule.as_str().to_string(),
                Rule::constant_expression => expr = Some(self.parse_constant_expression(rule)?),
                _ => unreachable!(),
            }
        }

        let kind = if str.starts_with("case") {
            StmtKind::Case {
                expr: expr.unwrap(),
                stmt: None,
            }
        } else if str.starts_with("default") {
            StmtKind::Default(None)
        } else {
            StmtKind::Label { name, stmt: None }
        };
        Ok(Rc::new(RefCell::new(Stmt {
            attributes,
            kind,
            ..Stmt::new(self.file_id, from_pest_span(span))
        })))
    }

    pub fn parse_selection_statement(
        &self,
        rule: Pair<Rule>,
    ) -> Result<Rc<RefCell<Stmt>>, Diagnostic<usize>> {
        let span = rule.as_span();
        let is_if = rule.as_str().starts_with("if");
        let mut condition = None;
        let mut body = None;
        let mut else_body = None;

        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::expression => condition = Some(self.parse_expression(rule)?),
                Rule::statement => {
                    if let None = body {
                        body = Some(self.parse_statement(rule)?);
                    } else {
                        else_body = Some(self.parse_statement(rule)?);
                    }
                }
                _ => unreachable!(),
            }
        }
        let condition = condition.unwrap();
        let body = body.unwrap();
        Ok(Rc::new(RefCell::new(Stmt {
            kind: if is_if {
                StmtKind::If {
                    condition,
                    body,
                    else_body,
                }
            } else {
                StmtKind::Switch { condition, body }
            },
            ..Stmt::new(self.file_id, from_pest_span(span))
        })))
    }

    pub fn parse_iteration_statement(
        &self,
        rule: Pair<Rule>,
    ) -> Result<Rc<RefCell<Stmt>>, Diagnostic<usize>> {
        let span = rule.as_span();
        let str = rule.as_str();
        let mut stmts = Vec::new();
        let mut exprs = Vec::new();
        let mut decls = Vec::new();

        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::expression => exprs.push(self.parse_expression(rule)?),
                Rule::statement => stmts.push(self.parse_statement(rule)?),
                Rule::declaration => decls.extend(self.parse_declaration(rule)?),
                _ => unreachable!(),
            }
        }
        //逆序排列, 方便后面处理
        exprs.reverse();
        decls.reverse();
        stmts.reverse();

        Ok(Rc::new(RefCell::new(Stmt {
            kind: if str.starts_with("while") {
                StmtKind::While {
                    condition: exprs.pop().unwrap(),
                    body: stmts.pop().unwrap(),
                }
            } else if str.starts_with("do") {
                StmtKind::DoWhile {
                    condition: exprs.pop().unwrap(),
                    body: stmts.pop().unwrap(),
                }
            } else {
                StmtKind::For {
                    init_expr: exprs.pop(),
                    init_decl: decls.pop(),
                    condition: exprs.pop(),
                    iter_expr: exprs.pop(),
                    body: stmts.pop().unwrap(),
                }
            },
            ..Stmt::new(self.file_id, from_pest_span(span))
        })))
    }

    pub fn parse_jump_statement(
        &self,
        rule: Pair<Rule>,
    ) -> Result<Rc<RefCell<Stmt>>, Diagnostic<usize>> {
        let span = rule.as_span();
        let str = rule.as_str();
        let mut name = String::new();
        let mut expr = None;

        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::identifier => name = rule.as_str().to_string(),
                Rule::expression => expr = Some(self.parse_expression(rule)?),
                _ => unreachable!(),
            }
        }
        Ok(Rc::new(RefCell::new(Stmt {
            kind: if str.starts_with("goto") {
                StmtKind::Goto(name)
            } else if str.starts_with("continue") {
                StmtKind::Continue
            } else if str.starts_with("break") {
                StmtKind::Break
            } else {
                StmtKind::Return { expr }
            },
            ..Stmt::new(self.file_id, from_pest_span(span))
        })))
    }

    pub fn parse_labeled_statement(
        &self,
        rule: Pair<Rule>,
    ) -> Result<Rc<RefCell<Stmt>>, Diagnostic<usize>> {
        let mut label = None;
        let mut stmt = None;
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::label => label = Some(self.parse_label(rule)?),
                Rule::statement => stmt = Some(self.parse_statement(rule)?),
                _ => unreachable!(),
            }
        }
        let label = label.unwrap();
        if let StmtKind::Label { name: _, stmt: old } = &mut label.borrow_mut().kind {
            *old = stmt;
        } else if let StmtKind::Case { expr: _, stmt: old } = &mut label.borrow_mut().kind {
            *old = stmt;
        } else if let StmtKind::Default(old) = &mut label.borrow_mut().kind {
            *old = stmt;
        }
        Ok(label)
    }

    pub fn parse_statement(
        &self,
        rule: Pair<Rule>,
    ) -> Result<Rc<RefCell<Stmt>>, Diagnostic<usize>> {
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::labeled_statement => return Ok(self.parse_labeled_statement(rule)?),
                Rule::unlabeled_statement => return Ok(self.parse_unlabeled_statement(rule)?),
                _ => unreachable!(),
            }
        }
        unreachable!()
    }
}
