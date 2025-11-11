pub mod cmacro;
pub mod macro_replace;
#[cfg(test)]
pub mod tests;

use crate::diagnostic::warning;
use cmacro::{Macro, PlaceMarker};
use pest::Parser;
use pest::error::{Error, ErrorVariant};
use pest::iterators::Pair;
use pest_derive::Parser;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

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

    pub fn process_source_file_inclusion(
        &mut self,
        rule: &Pair<Rule>,
    ) -> Result<String, Error<Rule>> {
        for rule in rule.clone().into_inner() {
            match rule.as_rule() {
                Rule::header_name => {
                    let file_name = rule.as_str();
                    let mut search_path = self
                        .include_path
                        .iter()
                        .map(|x| PathBuf::from(x))
                        .collect::<Vec<PathBuf>>();
                    if file_name.starts_with('"') {
                        if let Some(t) = Path::new(&self.file_path).parent() {
                            search_path.push(PathBuf::from(t));
                        }
                    }
                    let file_name = &file_name[1..file_name.len() - 1].to_string(); //去掉前后的<>或"
                    for path in search_path {
                        let file_path = path.join(&file_name);
                        let file_content = match fs::read_to_string(&file_path) {
                            Ok(t) => t,
                            Err(e) => {
                                return Err(Error::new_from_span(
                                    ErrorVariant::CustomError {
                                        message: format!(
                                            "Cannot include file '{}': {}",
                                            file_name, e
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
                }
                _ => {}
            }
        }
        Ok("\n".to_string())
    }
}
