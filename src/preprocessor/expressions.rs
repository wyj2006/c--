///包括了与表达式有关的方法
use super::{Preprocessor, Rule};
use crate::ast::expr::ExprKind;
use crate::diagnostic::{from_pest_span, map_pest_err};
use crate::file_map::source_map;
use crate::parser;
use crate::parser::CParser;
use crate::preprocessor::cmacro::{STDC_EMBED_EMPTY, STDC_EMBED_FOUND, STDC_EMBED_NOT_FOUND};
use crate::symtab::SymbolTable;
use crate::typechecker::TypeChecker;
use crate::variant::Variant;
use codespan_reporting::diagnostic::{Diagnostic, Label};
use num::ToPrimitive;
use pest::Parser;
use pest::iterators::{Pair, Pairs};
use pest::pratt_parser::{Assoc, Op, PrattParser};
use std::cell::RefCell;
use std::fs;
use std::rc::Rc;
use std::sync::LazyLock;

static PRATT_PARSER: LazyLock<PrattParser<Rule>> = LazyLock::new(|| {
    PrattParser::new()
        .op(Op::infix(Rule::or, Assoc::Left))
        .op(Op::infix(Rule::and, Assoc::Left))
        .op(Op::infix(Rule::bit_or, Assoc::Left))
        .op(Op::infix(Rule::bit_xor, Assoc::Left))
        .op(Op::infix(Rule::bit_and, Assoc::Left))
        .op(Op::infix(Rule::eq, Assoc::Left) | Op::infix(Rule::neq, Assoc::Left))
        .op(Op::infix(Rule::lt, Assoc::Left)
            | Op::infix(Rule::le, Assoc::Left)
            | Op::infix(Rule::gt, Assoc::Left)
            | Op::infix(Rule::ge, Assoc::Left))
        .op(Op::infix(Rule::lshift, Assoc::Left) | Op::infix(Rule::rshift, Assoc::Left))
        .op(Op::infix(Rule::add, Assoc::Left) | Op::infix(Rule::sub, Assoc::Left))
        .op(Op::infix(Rule::mul, Assoc::Left)
            | Op::infix(Rule::div, Assoc::Left)
            | Op::infix(Rule::r#mod, Assoc::Left))
        .op(Op::prefix(Rule::positve)
            | Op::prefix(Rule::negative)
            | Op::prefix(Rule::bit_not)
            | Op::prefix(Rule::not))
});

impl Preprocessor {
    ///计算constant_expression的值
    pub fn process_constant_expression(
        &mut self,
        rule: Pair<Rule>,
    ) -> Result<isize, Diagnostic<usize>> {
        let mut condition = 0;
        let mut true_value = None;
        let mut false_value = None;
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::expression => condition = self.process_expression(rule.into_inner())?,
                Rule::constant_expression => {
                    if let None = true_value {
                        true_value = Some(self.process_constant_expression(rule)?);
                    } else {
                        false_value = Some(self.process_constant_expression(rule)?);
                    }
                }
                _ => {}
            }
        }
        if let Some(true_v) = true_value
            && let Some(false_v) = false_value
        {
            if condition != 0 {
                Ok(true_v)
            } else {
                Ok(false_v)
            }
        } else {
            Ok(condition)
        }
    }

    ///计算expression的值
    pub fn process_expression(&mut self, rules: Pairs<Rule>) -> Result<isize, Diagnostic<usize>> {
        PRATT_PARSER
            .map_primary(|primary| match primary.as_rule() {
                Rule::integer_constant => match primary.as_str().parse::<isize>() {
                    Ok(t) => Ok(t),
                    Err(e) => Err(Diagnostic::error()
                        .with_message(format!("invalid integer '{}': {}", primary.as_str(), e))
                        .with_label(Label::primary(
                            self.file_id,
                            from_pest_span(primary.as_span()),
                        ))),
                },
                Rule::character_constant => {
                    let part_id =
                        source_map(self.file_path(), &self.to_tokens(primary.clone(), false));
                    let expr = CParser::new(part_id)
                        .parse_character_constant(
                            map_pest_err(
                                part_id,
                                CParser::parse(parser::Rule::character_constant, primary.as_str()),
                            )?
                            .next()
                            .unwrap(),
                        )
                        .unwrap();
                    let mut type_checker =
                        TypeChecker::new(Rc::new(RefCell::new(SymbolTable::new())));
                    type_checker.visit_expr(Rc::clone(&expr))?;
                    Ok(match &expr.borrow().value {
                        Variant::Int(value) => value.to_isize().unwrap_or(0),
                        _ => 0,
                    })
                }
                Rule::constant_expression => self.process_constant_expression(primary),
                Rule::identifier => Ok(0),
                Rule::defined_macro_expression => {
                    let mut macro_name = String::new();
                    let mut macro_span = primary.as_span();
                    for rule in primary.into_inner() {
                        match rule.as_rule() {
                            Rule::identifier => {
                                macro_name = rule.as_str().to_string();
                                macro_span = rule.as_span();
                            }
                            _ => {}
                        }
                    }
                    if let Some(_) = self.find_macro(&macro_name, from_pest_span(macro_span)) {
                        Ok(1)
                    } else {
                        Ok(0)
                    }
                }
                Rule::has_include_expression => {
                    let mut header_name = None;
                    for rule in primary.into_inner() {
                        match rule.as_rule() {
                            Rule::header_name => header_name = Some(rule.as_str().to_string()),
                            Rule::string_literal => {
                                header_name = Some(self.process_string_literal(rule)?)
                            }
                            _ => {}
                        }
                    }
                    if let Some(header_name) = header_name {
                        Ok((self.get_possible_filepath(header_name.as_str()).len() > 0) as isize)
                    } else {
                        Ok(0)
                    }
                }
                Rule::has_embed_expression => {
                    let mut header_name = None;
                    for rule in primary.into_inner() {
                        match rule.as_rule() {
                            Rule::header_name => header_name = Some(rule.as_str().to_string()),
                            Rule::string_literal => {
                                header_name = Some(self.process_string_literal(rule)?)
                            }
                            Rule::embed_parameter_sequence => {
                                if let Err(_) = self.process_embed_parameters(rule) {
                                    return Ok(STDC_EMBED_NOT_FOUND);
                                }
                            }
                            _ => {}
                        }
                    }
                    if let Some(header_name) = header_name {
                        for file_path in self.get_possible_filepath(header_name.as_str()) {
                            if let Ok(t) = fs::metadata(file_path) {
                                if t.len() == 0 {
                                    return Ok(STDC_EMBED_EMPTY);
                                } else {
                                    return Ok(STDC_EMBED_FOUND);
                                }
                            }
                        }
                    }
                    Ok(STDC_EMBED_NOT_FOUND)
                }
                Rule::has_c_attribute => {
                    let mut prefix_name = None;
                    let mut name = String::new();

                    for rule in primary.into_inner() {
                        match rule.as_rule() {
                            Rule::attribute_prefix => prefix_name = Some(rule.as_str().to_string()),
                            Rule::identifier => name = rule.as_str().to_string(),
                            _ => unreachable!(),
                        }
                    }

                    if name.starts_with("__") && name.ends_with("__") {
                        //去除两边的下划线
                        name = name[2..name.len() - 2].to_string();
                    }

                    Ok(has_c_attribute(prefix_name, name))
                }
                Rule::predefined_constant => {
                    if primary.as_str() == "true" {
                        Ok(1)
                    } else {
                        Ok(0)
                    }
                }
                _ => unreachable!(),
            })
            .map_prefix(|op, rhs| {
                let rhs_value = match rhs {
                    Ok(t) => t,
                    Err(e) => return Err(e),
                };
                match op.as_rule() {
                    Rule::positve => Ok(rhs_value),
                    Rule::negative => Ok(-rhs_value),
                    Rule::bit_not => Ok(!rhs_value),
                    Rule::not => Ok((rhs_value == 0) as isize),
                    _ => unreachable!(),
                }
            })
            .map_infix(|lhs, op, rhs| {
                let lhs_value = match lhs {
                    Ok(t) => t,
                    Err(e) => return Err(e),
                };
                let rhs_value = match rhs {
                    Ok(t) => t,
                    Err(e) => return Err(e),
                };
                match op.as_rule() {
                    Rule::or => Ok((lhs_value != 0 || rhs_value != 0) as isize),
                    Rule::and => Ok((lhs_value != 0 && rhs_value != 0) as isize),
                    Rule::bit_or => Ok(lhs_value | rhs_value),
                    Rule::bit_xor => Ok(lhs_value ^ rhs_value),
                    Rule::bit_and => Ok(lhs_value & rhs_value),
                    Rule::eq => Ok((lhs_value == rhs_value) as isize),
                    Rule::neq => Ok((lhs_value != rhs_value) as isize),
                    Rule::lt => Ok((lhs_value < rhs_value) as isize),
                    Rule::le => Ok((lhs_value <= rhs_value) as isize),
                    Rule::gt => Ok((lhs_value > rhs_value) as isize),
                    Rule::ge => Ok((lhs_value >= rhs_value) as isize),
                    Rule::rshift => Ok(lhs_value >> rhs_value),
                    Rule::lshift => Ok(lhs_value << rhs_value),
                    Rule::add => Ok(lhs_value + rhs_value),
                    Rule::sub => Ok(lhs_value - rhs_value),
                    Rule::mul => Ok(lhs_value * rhs_value),
                    Rule::div => Ok(lhs_value / rhs_value),
                    Rule::r#mod => Ok(lhs_value % rhs_value),
                    _ => unreachable!(),
                }
            })
            .parse(rules)
    }

    pub fn process_string_literal(&self, rule: Pair<Rule>) -> Result<String, Diagnostic<usize>> {
        let part_id = source_map(self.file_path(), &self.to_tokens(rule.clone(), false));
        let expr = CParser::new(part_id)
            .parse_string_literal(
                map_pest_err(
                    part_id,
                    CParser::parse(parser::Rule::string_literal, rule.as_str()),
                )?
                .next()
                .unwrap(),
            )
            .unwrap();
        Ok(match &expr.borrow().kind {
            ExprKind::String { text, .. } => text.clone(),
            _ => unreachable!(),
        })
    }
}

pub fn has_c_attribute(prefix_name: Option<String>, name: String) -> isize {
    match (prefix_name.as_deref(), name.as_str()) {
        (None, "deprecated") => 201904,
        (None, "fallthrough") => 201904,
        (None, "maybe_unused") => 201904,
        (None, "nodiscard") => 202003,
        (None, "noreturn") => 202202,
        (None, "unsequenced") => 202207,
        (None, "reproducible") => 202207,
        _ => 0,
    }
}
