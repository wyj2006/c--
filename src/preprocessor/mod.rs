pub mod cmacro;
pub mod macro_replace;
#[cfg(test)]
pub mod tests;

use cmacro::{Macro, PlaceMarker};
use pest::Parser;
use pest::error::{Error, ErrorVariant};
use pest::iterators::Pair;
use pest_derive::Parser;
use std::collections::HashMap;

#[derive(Parser)]
#[grammar = "src/grammar/lexer.pest"]
#[grammar = "src/grammar/preprocessor.pest"]
pub struct Preprocessor {
    pub file_path: String,
    pub file_content: String,
    pub user_macro: HashMap<String, Macro>,
}

impl Preprocessor {
    pub fn new(file_path: String, file_content: String) -> Preprocessor {
        Preprocessor {
            file_path,
            file_content,
            user_macro: HashMap::new(),
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
                    //预处理指令后也有一个换行符
                    result += "\n";
                }
                Rule::text_line => {
                    result += &PlaceMarker::vec_tostring(
                        self.replace_macro(PlaceMarker::vec_from(&rule))?,
                    );
                    //每个text_line后面都有一个换行符
                    //只是不在rule中
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
                //只有define和undef不允许替换后再处理
                if let Rule::directive_keyword = pair.as_rule()
                    && (pair.as_str() == "define" || pair.as_str() == "undef")
                {
                    input = rule.as_str().to_string();
                    break;
                }
            }
        }
        if input == "" {
            input = PlaceMarker::vec_tostring(self.replace_macro(PlaceMarker::vec_from(&rule))?);
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
        let mut parameters = Vec::new();
        let mut has_varparam = false;
        let mut replace_list = Vec::new();
        for rule in rule.clone().into_inner() {
            match rule.as_rule() {
                Rule::identifier => {
                    name = rule.as_str().to_string();
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
                    //TODO 处理与__VA_ARGS__和__VA_OPT__和#和##有关的错误或者让PlaceMarker带上位置信息
                    replace_list = PlaceMarker::vec_from(&rule);
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
        if let Some(pre_macro) = self.find_macro(&name) {
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
        Ok(String::new())
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
        Ok(String::new())
    }
}
