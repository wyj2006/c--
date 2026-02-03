use super::Rule;
use crate::diagnostic::from_pest_span;
use codespan::Span;
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
    Text(String, bool, Span), //bool用于判断这是否是空白字符
    Identifier(String, Span),
    StringizeStart(Span),
    StringizeEnd(Span),
    Contactor(Span),
    VaOptStart(bool, Span), //bool用来表示VaOptStart到VaOptEnd之间的内容是否保留
    VaOptEnd(Span),
    //相互配对的CallStart和CallEnd内容应该一样
    CallStart(String, Span),
    CallEnd(String, Span),
    //CallArgStart和CallArgEnd同理
    CallArgStart(String, u128, Span),
    CallArgEnd(String, u128, Span),
    //'#'的运算结果
    Stringized(Vec<PlaceMarker>, Span),
    //'##'的运算结果
    Contacted(Box<PlaceMarker>, Box<PlaceMarker>, Span),
}

impl PlaceMarker {
    ///transform_hashhash: 是否将'##'转换成Contactor
    pub fn vec_from<'a>(rule: Pair<'a, Rule>, transform_hashhash: bool) -> Vec<PlaceMarker> {
        let mut placemarkers: Vec<PlaceMarker> = Vec::new();
        match rule.as_rule() {
            Rule::replacement_list => {
                for rule in rule.into_inner() {
                    placemarkers.extend(PlaceMarker::vec_from(rule, true));
                }
            }
            Rule::text_line | Rule::pp_tokens | Rule::up_directives => {
                for rule in rule.into_inner() {
                    placemarkers.extend(PlaceMarker::vec_from(rule, transform_hashhash));
                }
            }
            Rule::punctuator if rule.as_str() == "##" && transform_hashhash => {
                placemarkers.push(PlaceMarker::Contactor(from_pest_span(rule.as_span())));
            }
            Rule::identifier => {
                placemarkers.push(PlaceMarker::Identifier(
                    rule.as_str().to_string(),
                    from_pest_span(rule.as_span()),
                ));
            }
            Rule::WHITESPACE => {
                placemarkers.push(PlaceMarker::Text(
                    rule.as_str().to_string(),
                    true,
                    from_pest_span(rule.as_span()),
                ));
            }
            _ => {
                placemarkers.push(PlaceMarker::Text(
                    rule.as_str().to_string(),
                    false,
                    from_pest_span(rule.as_span()),
                ));
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
                PlaceMarker::Text(a, _, _) => a.to_string(),
                PlaceMarker::Identifier(a, _) => a.to_string(),
                PlaceMarker::StringizeStart(_) => "#".to_string(),
                PlaceMarker::StringizeEnd(_)
                | PlaceMarker::VaOptStart(_, _)
                | PlaceMarker::VaOptEnd(_)
                | PlaceMarker::CallStart(_, _)
                | PlaceMarker::CallEnd(_, _)
                | PlaceMarker::CallArgStart(_, _, _)
                | PlaceMarker::CallArgEnd(_, _, _) => "".to_string(),
                PlaceMarker::Contactor(_) => "##".to_string(),
                PlaceMarker::Stringized(args, _) =>
                    format!("{:?}", PlaceMarker::vec_tostring(args.clone())),
                PlaceMarker::Contacted(left, right, _) => format!("{left}{right}"),
            }
        )
    }
}
