pub mod parse_decl;
pub mod parse_expr;
pub mod parse_stmt;
#[cfg(test)]
pub mod tests;

use crate::ast::Attribute;
use crate::ast::AttributeKind;
use crate::ast::Designation;
use crate::ast::DesignationKind;
use crate::ast::Initializer;
use crate::ast::InitializerKind;
use crate::ast::TranslationUnit;
use crate::ast::expr::ExprKind;
use crate::diagnostic::from_pest_span;
use crate::diagnostic::map_pest_err;
use crate::files;
use codespan_reporting::diagnostic::Diagnostic;
use codespan_reporting::diagnostic::Label;
use codespan_reporting::files::Files;
use pest::{Parser, iterators::Pair};
use pest_derive::Parser;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Parser)]
#[grammar = "src/grammar/lexer.pest"]
#[grammar = "src/grammar/parser.pest"]
pub struct CParser {
    pub file_id: usize,
}

impl CParser {
    pub fn new(file_id: usize) -> CParser {
        CParser { file_id }
    }

    pub fn parse_to_ast(&self) -> Result<Rc<RefCell<TranslationUnit>>, Diagnostic<usize>> {
        let source = files.lock().unwrap().source(self.file_id).unwrap();
        let rules = map_pest_err(
            self.file_id,
            CParser::parse(Rule::translation_unit, source.as_str()),
        )?;
        for rule in rules {
            match rule.as_rule() {
                Rule::translation_unit => return self.parse_translation_unit(rule),
                _ => unreachable!(),
            }
        }
        unreachable!()
    }

    pub fn parse_translation_unit(
        &self,
        rule: Pair<Rule>,
    ) -> Result<Rc<RefCell<TranslationUnit>>, Diagnostic<usize>> {
        let span = rule.as_span();
        let mut decls = Vec::new();
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::function_definition => decls.extend(self.parse_function_definition(rule)?),
                Rule::declaration => decls.extend(self.parse_declaration(rule)?),
                Rule::EOI => {}
                _ => unreachable!(),
            }
        }
        Ok(Rc::new(RefCell::new(TranslationUnit {
            decls,
            ..TranslationUnit::new(self.file_id, from_pest_span(span))
        })))
    }

    pub fn parse_attribute_specifier_sequence(
        &self,
        rule: Pair<Rule>,
    ) -> Result<Vec<Rc<RefCell<Attribute>>>, Diagnostic<usize>> {
        let mut attributes = Vec::new();
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::attribute => attributes.push(self.parse_attribute(rule)?),
                _ => unreachable!(),
            }
        }
        Ok(attributes)
    }

    pub fn parse_attribute_token(
        &self,
        rule: Pair<Rule>,
    ) -> Result<(Option<String>, String), Diagnostic<usize>> {
        let mut prefix_name = None;
        let mut name = String::new();

        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::attribute_prefix => prefix_name = Some(rule.as_str().to_string()),
                Rule::identifier => name = rule.as_str().to_string(),
                _ => unreachable!(),
            }
        }

        Ok((prefix_name, name))
    }

    pub fn parse_attribute(
        &self,
        rule: Pair<Rule>,
    ) -> Result<Rc<RefCell<Attribute>>, Diagnostic<usize>> {
        let span = rule.as_span();
        let mut prefix_name = None;
        let mut name = String::new();
        let mut arguments = None;
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::attribute_token => (prefix_name, name) = self.parse_attribute_token(rule)?,
                Rule::attribute_argument_clause => {
                    //去除两侧括号
                    arguments = Some(rule.as_str()[1..rule.as_str().len() - 1].to_string())
                }
                _ => unreachable!(),
            }
        }

        if name.starts_with("__") && name.ends_with("__") {
            //去除两边的下划线
            name = name[2..name.len() - 2].to_string();
        }

        let kind;

        match (prefix_name.as_deref(), name.as_str()) {
            (None, a @ ("deprecated" | "nodiscard")) => {
                let mut reason = None;
                if let Some(t) = arguments {
                    match CParser::parse(Rule::string_literal, t.as_str()) {
                        Ok(rules) => {
                            for rule in rules {
                                let expr = self.parse_string_literal(rule)?;
                                match &expr.borrow().kind {
                                    ExprKind::String { text, .. } => {
                                        reason = Some(text.to_string())
                                    }
                                    _ => unreachable!(),
                                }
                                break;
                            }
                        }
                        Err(_) => {
                            return Err(Diagnostic::error()
                                .with_message("expected string literal as argument of 'deprecated'")
                                .with_label(Label::primary(self.file_id, from_pest_span(span))));
                        }
                    }
                }
                kind = match a {
                    "deprecated" => AttributeKind::Deprecated { reason },
                    "nodiscard" => AttributeKind::Nodiscard { reason },
                    _ => unreachable!(),
                }
            }
            (None, "noreturn") => {
                kind = AttributeKind::Noreturn;
            }
            (None, "fallthrough") => {
                kind = AttributeKind::FallThrough;
            }
            (None, "maybe_unused") => {
                kind = AttributeKind::MaybeUnused;
            }
            (None, "unsequenced") => {
                kind = AttributeKind::Unsequenced;
            }
            (None, "reproducible") => {
                kind = AttributeKind::Reproducible;
            }
            _ => {
                kind = AttributeKind::Unkown {
                    arguments: arguments,
                }
            }
        }
        Ok(Rc::new(RefCell::new(Attribute {
            prefix_name,
            name,
            kind,
            ..Attribute::new(self.file_id, from_pest_span(span))
        })))
    }

    pub fn parse_initializer(
        &self,
        rule: Pair<Rule>,
    ) -> Result<Rc<RefCell<Initializer>>, Diagnostic<usize>> {
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::braced_initializer => return self.parse_braced_initializer(rule),
                Rule::assignment_expression => {
                    return Ok(Rc::new(RefCell::new(Initializer::new(
                        self.file_id,
                        from_pest_span(rule.as_span()),
                        InitializerKind::Expr(self.parse_assignment_expression(rule)?),
                    ))));
                }
                _ => unreachable!(),
            }
        }
        unreachable!()
    }

    pub fn parse_braced_initializer(
        &self,
        rule: Pair<Rule>,
    ) -> Result<Rc<RefCell<Initializer>>, Diagnostic<usize>> {
        let span = rule.as_span();
        let mut initializers = Vec::new();
        let mut designation = Vec::new();
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::designation => designation.extend(self.parse_designation(rule)?),
                Rule::initializer => {
                    let initializer = self.parse_initializer(rule)?;
                    initializer.borrow_mut().designation.extend(designation);
                    initializers.push(initializer);
                    designation = Vec::new();
                }
                _ => unreachable!(),
            }
        }
        Ok(Rc::new(RefCell::new(Initializer::new(
            self.file_id,
            from_pest_span(span),
            InitializerKind::Braced(initializers),
        ))))
    }

    pub fn parse_designation(
        &self,
        rule: Pair<Rule>,
    ) -> Result<Vec<Designation>, Diagnostic<usize>> {
        let span = rule.as_span();
        let mut designations = Vec::new();
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::constant_expression => designations.push(Designation::new(
                    self.file_id,
                    from_pest_span(span),
                    DesignationKind::Subscript(self.parse_constant_expression(rule)?),
                )),
                Rule::identifier => {
                    designations.push(Designation::new(
                        self.file_id,
                        from_pest_span(span),
                        DesignationKind::MemberAccess(rule.as_str().to_string()),
                    ));
                }
                _ => unreachable!(),
            }
        }
        Ok(designations)
    }
}
