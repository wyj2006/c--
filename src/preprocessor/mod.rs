pub mod cmacro;
pub mod macro_replace;
#[cfg(test)]
pub mod tests;

use crate::diagnostic::warning;
use cmacro::{Macro, PlaceMarker};
use pest::Parser;
use pest::error::{Error, ErrorVariant};
use pest::iterators::{Pair, Pairs};
use pest::pratt_parser::{Assoc, Op, PrattParser};
use pest_derive::Parser;
use std::cmp::min;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::{fs, usize};

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

#[derive(Parser)]
#[grammar = "src/grammar/lexer.pest"]
#[grammar = "src/grammar/preprocessor.pest"]
pub struct Preprocessor {
    pub file_path: String,
    pub file_content: String,
    pub user_macro: HashMap<String, Macro>,
    pub line_offset: isize,
    pub include_path: Vec<String>,
}

impl Preprocessor {
    pub fn new(file_path: String, file_content: String) -> Preprocessor {
        Preprocessor {
            file_path,
            file_content,
            user_macro: HashMap::new(),
            line_offset: 0,
            include_path: Vec::new(),
        }
    }

    pub fn process(&mut self) -> Result<String, Error<Rule>> {
        let input = self.file_content.clone();
        let rules = Preprocessor::parse(Rule::preprocessing_file, &input)?;
        let mut result = String::new();
        for rule in rules {
            if let Rule::preprocessing_file = rule.as_rule() {
                for rule in rule.into_inner() {
                    if let Rule::group_part = rule.as_rule() {
                        result += &self.process_group(&rule)?;
                    }
                }
            }
        }
        Ok(result)
    }

    fn process_group(&mut self, rule: &Pair<Rule>) -> Result<String, Error<Rule>> {
        let mut result = String::new();
        for rule in rule.clone().into_inner() {
            match rule.as_rule() {
                Rule::up_directives => {
                    result += &self.process_directive(&rule)?;
                }
                Rule::text_line => {
                    result += &PlaceMarker::vec_tostring(
                        self.replace_macro(PlaceMarker::vec_from(&rule, false), &mut Vec::new())?,
                    );
                    //每个text_line后面都有一个换行符, 只是不在rule中
                    result += "\n";
                }
                _ => {}
            }
        }
        Ok(result)
    }

    fn process_directive(&mut self, rule: &Pair<Rule>) -> Result<String, Error<Rule>> {
        let mut input = String::new();
        if let Rule::up_directives = rule.as_rule() {
            for pair in rule.clone().into_inner() {
                //不允许替换后再处理的指令
                if let Rule::directive_keyword = pair.as_rule()
                    && !(vec!["include", "embed", "line"].contains(&pair.as_str()))
                {
                    input = rule.as_str().to_string();
                    break;
                }
            }
        }
        if input == "" {
            input = PlaceMarker::vec_tostring(
                self.replace_macro(PlaceMarker::vec_from(&rule, false), &mut Vec::new())?,
            ) + "\n";
        }
        let rules = Preprocessor::parse(Rule::directives, &input)?;
        let mut result = String::new();
        for rule in rules {
            if let Rule::directives = rule.as_rule() {
                for rule in rule.into_inner() {
                    match rule.as_rule() {
                        Rule::macro_define => {
                            result += &self.process_macro_define(&rule)?;
                        }
                        Rule::macro_undef => {
                            result += &self.process_macro_undef(&rule)?;
                        }
                        Rule::warning_directive => {
                            result += &self.process_warning(&rule)?;
                        }
                        Rule::error_directive => {
                            result += &self.process_error(&rule)?;
                        }
                        Rule::line_control => {
                            result += &self.process_line_control(&rule)?;
                        }
                        Rule::source_file_inclusion => {
                            result += &self.process_source_file_inclusion(&rule)?;
                        }
                        Rule::binary_resource_inclusion => {
                            result += &self.process_binary_resource_inclusion(&rule)?;
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(result)
    }

    pub fn process_macro_define(&mut self, rule: &Pair<Rule>) -> Result<String, Error<Rule>> {
        let mut object_like = true;
        let mut name = String::new();
        let mut name_span = rule.as_span();
        let mut parameters = Vec::new();
        let mut has_varparam = false;
        let mut replace_list = Vec::new();
        for rule in rule.clone().into_inner() {
            match rule.as_rule() {
                Rule::identifier => {
                    name = rule.as_str().to_string();
                    name_span = rule.as_span();
                }
                Rule::function_identifier => {
                    object_like = false;
                    name = rule.as_str().to_string();
                }
                Rule::identifier_list => {
                    for rule in rule.into_inner() {
                        if let Rule::identifier = rule.as_rule() {
                            parameters.push(rule.as_str().to_string());
                        }
                    }
                }
                Rule::replacement_list => {
                    replace_list = PlaceMarker::vec_from(&rule, false);
                }
                Rule::varparam_symbol => {
                    has_varparam = true;
                }
                _ => {}
            }
        }
        let cmacro;
        if object_like {
            cmacro = Macro::Object {
                name: name.clone(),
                replace_list,
            };
        } else {
            name = name[..name.len() - 1].to_string(); //去除'('
            cmacro = Macro::Function {
                name: name.clone(),
                parameters,
                has_varparam,
                replace_list,
            };
        }
        if let Some(pre_macro) = self.find_macro(&name, name_span) {
            if pre_macro != cmacro {
                return Err(Error::new_from_span(
                    ErrorVariant::CustomError {
                        message: format!("macro '{}' redefined", name),
                    },
                    rule.as_span(),
                ));
            }
        } else {
            self.user_macro.insert(name, cmacro);
        }
        Ok("\n".to_string())
    }

    pub fn process_macro_undef(&mut self, rule: &Pair<Rule>) -> Result<String, Error<Rule>> {
        let mut name = String::new();
        for rule in rule.clone().into_inner() {
            match rule.as_rule() {
                Rule::identifier => name = rule.as_str().to_string(),
                _ => {}
            }
        }
        self.user_macro.remove(&name);
        Ok("\n".to_string())
    }

    pub fn process_warning(&mut self, rule: &Pair<Rule>) -> Result<String, Error<Rule>> {
        let mut message = String::new();
        for rule in rule.clone().into_inner() {
            match rule.as_rule() {
                Rule::pp_tokens => message = rule.as_str().to_string(),
                _ => {}
            }
        }
        warning::<Rule>(message, rule.as_span(), &self.file_path);
        Ok("\n".to_string())
    }

    pub fn process_error(&mut self, rule: &Pair<Rule>) -> Result<String, Error<Rule>> {
        let mut message = String::new();
        for rule in rule.clone().into_inner() {
            match rule.as_rule() {
                Rule::pp_tokens => message = rule.as_str().to_string(),
                _ => {}
            }
        }
        Err(
            Error::new_from_span(ErrorVariant::CustomError { message }, rule.as_span())
                .with_path(&self.file_path),
        )
    }

    pub fn process_line_control(&mut self, rule: &Pair<Rule>) -> Result<String, Error<Rule>> {
        for rule in rule.clone().into_inner() {
            match rule.as_rule() {
                Rule::digit_sequence => {
                    self.line_offset = rule.as_str().parse::<isize>().unwrap()
                        - rule.as_span().start_pos().line_col().0 as isize
                        - 1
                }
                Rule::string_literal => {
                    //TODO 正确解析字符串
                    self.file_path = rule.as_str()[1..rule.as_str().len() - 1].to_string();
                }
                _ => {}
            }
        }
        Ok("\n".to_string())
    }

    pub fn get_possible_filepath(&self, header_name: &str) -> Vec<PathBuf> {
        let mut search_path = self
            .include_path
            .iter()
            .map(|x| PathBuf::from(x))
            .collect::<Vec<PathBuf>>();
        let mut possible_path = Vec::new();
        if header_name.starts_with('"') {
            if let Some(t) = Path::new(&self.file_path).parent() {
                search_path.push(PathBuf::from(t));
            }
        }
        let file_name = &header_name[1..header_name.len() - 1].to_string(); //去掉前后的<>或"
        for path in search_path {
            let file_path = path.join(&file_name);
            if file_path.exists() {
                possible_path.push(file_path)
            }
        }
        possible_path
    }

    pub fn process_source_file_inclusion(
        &mut self,
        rule: &Pair<Rule>,
    ) -> Result<String, Error<Rule>> {
        for rule in rule.clone().into_inner() {
            match rule.as_rule() {
                Rule::header_name => {
                    for file_path in self.get_possible_filepath(rule.as_str()) {
                        let file_content = match fs::read_to_string(&file_path) {
                            Ok(t) => t,
                            Err(e) => {
                                return Err(Error::new_from_span(
                                    ErrorVariant::CustomError {
                                        message: format!(
                                            "Error occurred when opening or reading: {}",
                                            e
                                        ),
                                    },
                                    rule.as_span(),
                                ));
                            }
                        };
                        let mut preprocessor = Preprocessor::new(
                            file_path.to_string_lossy().to_string(),
                            file_content,
                        );
                        let result = match preprocessor.process() {
                            Ok(t) => t,
                            Err(e) => return Err(e),
                        };
                        return Ok(result + "\n");
                    }
                    return Err(Error::new_from_span(
                        ErrorVariant::CustomError {
                            message: format!(
                                "File '{}' not found",
                                rule.as_str()[1..rule.as_str().len() - 1].to_string()
                            ),
                        },
                        rule.as_span(),
                    ));
                }
                _ => {}
            }
        }
        Ok("\n".to_string())
    }

    pub fn process_binary_resource_inclusion(
        &mut self,
        rule: &Pair<Rule>,
    ) -> Result<String, Error<Rule>> {
        let mut header_name = "";
        let mut limit = usize::MAX;
        let mut prefix = String::new();
        let mut suffix = String::new();
        let mut if_empty = String::new();

        for rule in rule.clone().into_inner() {
            match rule.as_rule() {
                Rule::header_name => {
                    header_name = rule.as_str();
                }
                Rule::embed_parameter_sequence => {
                    (limit, prefix, suffix, if_empty) = self.process_embed_parameters(&rule)?;
                }
                _ => {}
            }
        }

        for file_path in self.get_possible_filepath(header_name) {
            let mut buf = [0; 1024];
            let file = match File::open(file_path) {
                Ok(t) => t,
                Err(e) => {
                    return Err(Error::new_from_span(
                        ErrorVariant::CustomError {
                            message: format!("Error occurred when opening: {}", e),
                        },
                        rule.as_span(),
                    ));
                }
            };
            let mut reader = BufReader::new(file);
            let mut data: Vec<u8> = Vec::new();
            loop {
                let n = match reader.read(&mut buf) {
                    Ok(t) => t,
                    Err(e) => {
                        return Err(Error::new_from_span(
                            ErrorVariant::CustomError {
                                message: format!("Error occurred when reading: {}", e),
                            },
                            rule.as_span(),
                        ));
                    }
                };
                if n == 0 {
                    break;
                }
                data.extend(&buf[0..n]);
                if data.len() >= limit {
                    break;
                }
            }
            if data.len() > 0 {
                return Ok(prefix
                    + &data[0..min(data.len(), limit)]
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>()
                        .join(",")
                    + &suffix
                    + "\n");
            } else {
                return Ok(if_empty.to_string());
            }
        }

        Err(Error::new_from_span(
            ErrorVariant::CustomError {
                message: format!(
                    "File '{}' not found",
                    header_name[1..header_name.len() - 1].to_string()
                ),
            },
            rule.as_span(),
        ))
    }

    pub fn process_embed_parameters(
        &mut self,
        rule: &Pair<Rule>,
    ) -> Result<(usize, String, String, String), Error<Rule>> {
        let mut limit = usize::MAX;
        let mut prefix = "";
        let mut suffix = "";
        let mut if_empty = "";

        for rule in rule.clone().into_inner() {
            match rule.as_rule() {
                Rule::pp_parameter => {
                    let mut param_name = "";
                    let mut param_clause = "";
                    for rule in rule.clone().into_inner() {
                        match rule.as_rule() {
                            Rule::pp_parameter_name => {
                                param_name = rule.as_str();
                            }
                            Rule::pp_parameter_clause => {
                                param_clause = rule.as_str();
                                param_clause = &param_clause[1..param_clause.len() - 1]; //去掉'(' ')'
                            }
                            _ => {}
                        }
                    }
                    match param_name {
                        "limit" | "__limit__" => {
                            limit = self.process_constant_expression(&Preprocessor::parse(
                                Rule::constant_expression,
                                param_clause,
                            )?)? as usize;
                        }
                        "prefix" | "__prefix__" => {
                            prefix = param_clause;
                        }
                        "suffix" | "__suffix__" => {
                            suffix = param_clause;
                        }
                        "if_empty" | "__if_empty__" => {
                            if_empty = param_clause;
                        }
                        _ => {
                            warning::<Rule>(
                                format!("Unkown parameter: {}", param_name),
                                rule.as_span(),
                                &self.file_path,
                            );
                        }
                    }
                }
                _ => {}
            }
        }

        Ok((
            limit,
            prefix.to_string(),
            suffix.to_string(),
            if_empty.to_string(),
        ))
    }

    ///计算constant_expression的值
    pub fn process_constant_expression(
        &mut self,
        rules: &Pairs<Rule>,
    ) -> Result<isize, Error<Rule>> {
        PRATT_PARSER
            .map_primary(|primary| match primary.as_rule() {
                Rule::integer_constant => match primary.as_str().parse() {
                    Ok(t) => Ok(t),
                    Err(e) => Err(Error::new_from_span(
                        ErrorVariant::CustomError {
                            message: format!("invalid integer '{}': {}", primary.as_str(), e),
                        },
                        primary.as_span(),
                    )),
                },
                //TODO 正确处理字符
                Rule::character_constant => Ok(0),
                Rule::constant_expression => {
                    self.process_constant_expression(&primary.into_inner())
                }
                Rule::identifier => Ok(0),
                Rule::defined_macro_expression => Err(Error::new_from_span(
                    ErrorVariant::CustomError {
                        message: format!("defined shall not appear within the constant expression"),
                    },
                    primary.as_span(),
                )),
                //TODO
                Rule::has_include_expression => Ok(0),
                Rule::has_embed_expression => Ok(0),
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
            .parse(rules.clone())
    }
}
