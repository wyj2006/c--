use super::Rule;
use pest::{Span, iterators::Pair};
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

//给placemarker使用的span
#[derive(Debug, PartialEq, Clone)]
pub struct PlaceMarkerSpan {
    pub input: String,
    pub start: usize,
    pub end: usize,
}

///作为宏替换的单元
#[derive(Debug, PartialEq, Clone)]
pub enum PlaceMarker {
    Text(String, bool, PlaceMarkerSpan), //bool用于判断这是否是空白字符
    Identifier(String, PlaceMarkerSpan),
    StringizeStart(PlaceMarkerSpan),
    StringizeEnd(PlaceMarkerSpan),
    Contactor(PlaceMarkerSpan),
    VaOptStart(bool, PlaceMarkerSpan), //bool用来表示VaOptStart到VaOptEnd之间的内容是否保留
    VaOptEnd(PlaceMarkerSpan),
    //相互配对的CallStart和CallEnd内容应该一样
    CallStart(String, PlaceMarkerSpan),
    CallEnd(String, PlaceMarkerSpan),
    //CallArgStart和CallArgEnd同理
    CallArgStart(String, u128, PlaceMarkerSpan),
    CallArgEnd(String, u128, PlaceMarkerSpan),
    //'#'的运算结果
    Stringized(Vec<PlaceMarker>, PlaceMarkerSpan),
    //'##'的运算结果
    Contacted(Box<PlaceMarker>, Box<PlaceMarker>, PlaceMarkerSpan),
}

impl PlaceMarker {
    ///transform_hashhash: 是否将'##'转换成Contactor
    pub fn vec_from(rule: &Pair<Rule>, transform_hashhash: bool) -> Vec<PlaceMarker> {
        let mut placemarkers: Vec<PlaceMarker> = Vec::new();
        let rule = rule.clone();
        match rule.as_rule() {
            Rule::replacement_list => {
                for rule in rule.into_inner() {
                    placemarkers.extend(PlaceMarker::vec_from(&rule, true));
                }
            }
            Rule::text_line | Rule::pp_tokens | Rule::up_directives => {
                for rule in rule.into_inner() {
                    placemarkers.extend(PlaceMarker::vec_from(&rule, transform_hashhash));
                }
            }
            Rule::punctuator if rule.as_str() == "##" && transform_hashhash => {
                placemarkers.push(PlaceMarker::Contactor(PlaceMarkerSpan::new_from_span(
                    rule.as_span(),
                )));
            }
            Rule::identifier => {
                placemarkers.push(PlaceMarker::Identifier(
                    rule.as_str().to_string(),
                    PlaceMarkerSpan::new_from_span(rule.as_span()),
                ));
            }
            Rule::WHITESPACE => {
                placemarkers.push(PlaceMarker::Text(
                    rule.as_str().to_string(),
                    true,
                    PlaceMarkerSpan::new_from_span(rule.as_span()),
                ));
            }
            _ => {
                placemarkers.push(PlaceMarker::Text(
                    rule.as_str().to_string(),
                    false,
                    PlaceMarkerSpan::new_from_span(rule.as_span()),
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

    pub fn span(&self) -> &PlaceMarkerSpan {
        match self {
            PlaceMarker::Text(_, _, a)
            | PlaceMarker::Identifier(_, a)
            | PlaceMarker::StringizeStart(a)
            | PlaceMarker::StringizeEnd(a)
            | PlaceMarker::VaOptStart(_, a)
            | PlaceMarker::VaOptEnd(a)
            | PlaceMarker::CallStart(_, a)
            | PlaceMarker::CallEnd(_, a)
            | PlaceMarker::CallArgStart(_, _, a)
            | PlaceMarker::CallArgEnd(_, _, a)
            | PlaceMarker::Contactor(a)
            | PlaceMarker::Stringized(_, a)
            | PlaceMarker::Contacted(_, _, a) => a,
        }
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

impl PlaceMarkerSpan {
    pub fn new_from_span(span: Span) -> PlaceMarkerSpan {
        PlaceMarkerSpan {
            input: span.get_input().to_string(),
            start: span.start(),
            end: span.end(),
        }
    }

    pub fn to_span(&self) -> Option<Span<'_>> {
        Span::new(self.input.as_str(), self.start, self.end)
    }
}
