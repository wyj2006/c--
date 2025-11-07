use super::Rule;
use pest::iterators::Pair;
use std::{cmp::PartialEq, fmt::Display};

#[derive(Debug, PartialEq, Clone)]
pub enum Macro {
    Object {
        name: String,
        replace_list: Vec<PlaceMarker>,
    },
    Function {
        name: String,
        parameters: Vec<String>,
        has_varparam: bool,
        replace_list: Vec<PlaceMarker>,
    },
}

///作为宏替换的单元
#[derive(Debug, PartialEq, Clone)]
pub enum PlaceMarker {
    Text(String, bool), //bool用于判断这是否是空白字符
    Identifier(String),
    StringizeStart,
    StringizeEnd,
    Contactor,
    VaOptStart(bool), //bool用来表示VaOptStart到VaOptEnd之间的内容是否保留
    VaOptEnd,
    //相互配对的CallStart和CallEnd内容应该一样
    CallStart(String),
    CallEnd(String),
    //CallArgStart和CallArgEnd同理
    CallArgStart(String, u128),
    CallArgEnd(String, u128),
}

impl PlaceMarker {
    pub fn vec_from(rule: &Pair<Rule>) -> Vec<PlaceMarker> {
        let mut placemarkers: Vec<PlaceMarker> = Vec::new();
        let mut final_rule = rule.clone();
        let mut final_rule_got = false;
        let mut transform_hashhash = false; //是否将'##'转换成Contactor
        //根据实际情况找到需要转换的rule
        while !final_rule_got {
            match final_rule.as_rule() {
                Rule::replacement_list => {
                    transform_hashhash = true;
                    final_rule = match final_rule.into_inner().next() {
                        Some(t) => t, //pp_tokens
                        None => {
                            //空的替换列表
                            return Vec::new();
                        }
                    };
                }
                Rule::text_line => {
                    final_rule = match final_rule.into_inner().next() {
                        Some(t) => t, //pp_tokens
                        None => {
                            //空行
                            return Vec::new();
                        }
                    };
                }
                Rule::pp_tokens => {
                    final_rule_got = true;
                }
                _ => {
                    return Vec::new();
                }
            }
        }
        for rule in final_rule.into_inner() {
            match rule.as_rule() {
                Rule::punctuator if rule.as_str() == "##" && transform_hashhash => {
                    placemarkers.push(PlaceMarker::Contactor);
                }
                Rule::identifier => {
                    placemarkers.push(PlaceMarker::Identifier(rule.as_str().to_string()));
                }
                Rule::WHITESPACE => {
                    placemarkers.push(PlaceMarker::Text(rule.as_str().to_string(), true));
                }
                _ => {
                    placemarkers.push(PlaceMarker::Text(rule.as_str().to_string(), false));
                }
            }
        }
        placemarkers
    }

    pub fn vec_tostring(placemarkers: Vec<PlaceMarker>) -> String {
        let mut result = String::new();
        for placemarker in placemarkers {
            result += &placemarker.to_string();
        }
        result
    }
}

impl Display for PlaceMarker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PlaceMarker::Text(a, _) => a.to_string(),
                PlaceMarker::Identifier(a) => a.to_string(),
                PlaceMarker::StringizeStart => "#".to_string(),
                PlaceMarker::StringizeEnd
                | PlaceMarker::VaOptStart(_)
                | PlaceMarker::VaOptEnd
                | PlaceMarker::CallStart(_)
                | PlaceMarker::CallEnd(_)
                | PlaceMarker::CallArgStart(_, _)
                | PlaceMarker::CallArgEnd(_, _) => "".to_string(),
                PlaceMarker::Contactor => "##".to_string(),
            }
        )
    }
}
