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
use crate::ctype::Type;
use crate::ctype::TypeKind;
use pest::{Parser, error::Error, iterators::Pair};
use pest_derive::Parser;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Parser)]
#[grammar = "src/grammar/lexer.pest"]
#[grammar = "src/grammar/parser.pest"]
pub struct CParser<'a> {
    pub file_content: &'a str,
}

impl<'a> CParser<'a> {
    pub fn new(file_content: &'a str) -> CParser<'a> {
        CParser { file_content }
    }

    pub fn parse_to_ast(&self) -> Result<Rc<RefCell<TranslationUnit<'a>>>, Error<Rule>> {
        let rules = CParser::parse(Rule::translation_unit, self.file_content)?;
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
        rule: Pair<'a, Rule>,
    ) -> Result<Rc<RefCell<TranslationUnit<'a>>>, Error<Rule>> {
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
        Ok(Rc::new(RefCell::new(TranslationUnit { span, decls })))
    }

    pub fn parse_attribute_specifier_sequence(
        &self,
        rule: Pair<'a, Rule>,
    ) -> Result<Vec<Rc<RefCell<Attribute<'a>>>>, Error<Rule>> {
        let mut attributes = Vec::new();
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::attribute => {
                    let span = rule.as_span();
                    let mut prefix_name = None;
                    let mut name = String::new();
                    let mut arguments = None;
                    for rule in rule.into_inner() {
                        match rule.as_rule() {
                            Rule::attribute_token => {
                                (prefix_name, name) = self.parse_attribute_token(rule)?
                            }
                            Rule::attribute_argument_clause => arguments = Some(rule),
                            _ => unreachable!(),
                        }
                    }
                    attributes.push(Rc::new(RefCell::new(Attribute {
                        span,
                        prefix_name,
                        name,
                        kind: AttributeKind::Unkown { arguments },
                    })));
                }
                _ => unreachable!(),
            }
        }
        Ok(attributes)
    }

    pub fn parse_attribute_token(
        &self,
        rule: Pair<'a, Rule>,
    ) -> Result<(Option<String>, String), Error<Rule>> {
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

    pub fn parse_initializer(
        &self,
        rule: Pair<'a, Rule>,
    ) -> Result<Rc<RefCell<Initializer<'a>>>, Error<Rule>> {
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::braced_initializer => return self.parse_braced_initializer(rule),
                Rule::assignment_expression => {
                    return Ok(Rc::new(RefCell::new(Initializer {
                        span: rule.as_span(),
                        designation: Vec::new(),
                        r#type: Rc::new(RefCell::new(Type {
                            span: rule.as_span(),
                            attributes: vec![],
                            kind: TypeKind::Error,
                        })),
                        kind: InitializerKind::Expr(self.parse_assignment_expression(rule)?),
                    })));
                }
                _ => unreachable!(),
            }
        }
        unreachable!()
    }

    pub fn parse_braced_initializer(
        &self,
        rule: Pair<'a, Rule>,
    ) -> Result<Rc<RefCell<Initializer<'a>>>, Error<Rule>> {
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
        Ok(Rc::new(RefCell::new(Initializer {
            span,
            designation: Vec::new(),
            kind: InitializerKind::Braced(initializers),
            r#type: Rc::new(RefCell::new(Type {
                span,
                attributes: vec![],
                kind: TypeKind::Error,
            })),
        })))
    }

    pub fn parse_designation(
        &self,
        rule: Pair<'a, Rule>,
    ) -> Result<Vec<Designation<'a>>, Error<Rule>> {
        let mut designations = Vec::new();
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::constant_expression => designations.push(Designation {
                    span: rule.as_span(),
                    kind: DesignationKind::Subscript(self.parse_constant_expression(rule)?),
                }),
                Rule::identifier => {
                    designations.push(Designation {
                        span: rule.as_span(),
                        kind: DesignationKind::MemberAccess(rule.as_str().to_string()),
                    });
                }
                _ => unreachable!(),
            }
        }
        Ok(designations)
    }
}
