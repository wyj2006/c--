use super::Rule;
use pest::{Span, iterators::Pair};
use std::{cmp::PartialEq, fmt::Display};

#[derive(Debug, PartialEq, Clone)]
pub enum Macro<'a> {
    Object {
        name: String,
        replace_list: Vec<PlaceMarker<'a>>,
    },
    Function {
        name: String,
        parameters: Vec<String>,
        has_varparam: bool,
        replace_list: Vec<PlaceMarker<'a>>,
    },
}

///作为宏替换的单元
#[derive(Debug, PartialEq, Clone)]
pub enum PlaceMarker<'a> {
    Text(String, bool, Span<'a>), //bool用于判断这是否是空白字符
    Identifier(String, Span<'a>),
    StringizeStart(Span<'a>),
    StringizeEnd(Span<'a>),
    Contactor(Span<'a>),
    VaOptStart(bool, Span<'a>), //bool用来表示VaOptStart到VaOptEnd之间的内容是否保留
    VaOptEnd(Span<'a>),
    //相互配对的CallStart和CallEnd内容应该一样
    CallStart(String, Span<'a>),
    CallEnd(String, Span<'a>),
    //CallArgStart和CallArgEnd同理
    CallArgStart(String, u128, Span<'a>),
    CallArgEnd(String, u128, Span<'a>),
    //'#'的运算结果
    Stringized(Vec<PlaceMarker<'a>>, Span<'a>),
    //'##'的运算结果
    Contacted(Box<PlaceMarker<'a>>, Box<PlaceMarker<'a>>, Span<'a>),
}

impl PlaceMarker<'_> {
    ///transform_hashhash: 是否将'##'转换成Contactor
    pub fn vec_from<'a>(rule: Pair<'a, Rule>, transform_hashhash: bool) -> Vec<PlaceMarker<'a>> {
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
                placemarkers.push(PlaceMarker::Contactor(rule.as_span()));
            }
            Rule::identifier => {
                placemarkers.push(PlaceMarker::Identifier(
                    rule.as_str().to_string(),
                    rule.as_span(),
                ));
            }
            Rule::WHITESPACE => {
                placemarkers.push(PlaceMarker::Text(
                    rule.as_str().to_string(),
                    true,
                    rule.as_span(),
                ));
            }
            _ => {
                placemarkers.push(PlaceMarker::Text(
                    rule.as_str().to_string(),
                    false,
                    rule.as_span(),
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

impl Display for PlaceMarker<'_> {
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
