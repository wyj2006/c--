use crate::{
    diagnostic::from_pest_span,
    file_map::source_lookup,
    preprocessor::{Preprocessor, Rule},
};
use codespan::Span;
use pest::iterators::Pair;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct Token {
    pub file_id: usize,
    pub span: Span,
    pub kind: TokenKind,
}

impl Token {
    pub fn new(file_id: usize, span: Span, kind: TokenKind) -> Token {
        let (file_id, span) = source_lookup(file_id, span);
        Token {
            file_id,
            span,
            kind,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Text {
        is_whitespace: bool,
        content: String,
    },
    Identifier {
        name: String,
    },
    StringizeStart,
    StringizeEnd,
    Contactor,
    VaOptStart {
        keep: bool,
    },
    VaOptEnd,
    CallStart {
        name: String,
    },
    CallEnd {
        name: String,
    },
    CallArgStart {
        belong: String,
        index: usize,
    },
    CallArgEnd {
        belong: String,
        index: usize,
    },
    Stringized(Vec<Token>),
    Contacted(Box<Token>, Box<Token>),
    Newline,
    EOI,
}

impl Preprocessor {
    pub fn to_tokens(&self, rule: Pair<Rule>, transform_hashhash: bool) -> Vec<Token> {
        let mut tokens: Vec<Token> = Vec::new();
        match rule.as_rule() {
            Rule::replacement_list => {
                for rule in rule.into_inner() {
                    tokens.extend(self.to_tokens(rule, true));
                }
            }
            Rule::text_line
            | Rule::pp_tokens
            | Rule::up_directives
            | Rule::pp_parameter_clause
            | Rule::pp_balanced_token_sequence
            | Rule::pp_balanced_token => {
                for rule in rule.into_inner() {
                    tokens.extend(self.to_tokens(rule, transform_hashhash));
                }
            }
            Rule::punctuator if rule.as_str() == "##" && transform_hashhash => {
                tokens.push(Token::new(
                    self.file_id,
                    from_pest_span(rule.as_span()),
                    TokenKind::Contactor,
                ));
            }
            Rule::identifier => {
                tokens.push(Token::new(
                    self.file_id,
                    from_pest_span(rule.as_span()),
                    TokenKind::Identifier {
                        name: rule.as_str().to_string(),
                    },
                ));
            }
            Rule::WHITESPACE => {
                tokens.push(Token::new(
                    self.file_id,
                    from_pest_span(rule.as_span()),
                    TokenKind::Text {
                        is_whitespace: true,
                        content: rule.as_str().to_string(),
                    },
                ));
            }
            Rule::newline => {
                tokens.push(Token::new(
                    self.file_id,
                    from_pest_span(rule.as_span()),
                    TokenKind::Newline,
                ));
            }
            Rule::EOI => {
                tokens.push(Token::new(
                    self.file_id,
                    from_pest_span(rule.as_span()),
                    TokenKind::EOI,
                ));
            }
            _ => {
                tokens.push(Token::new(
                    self.file_id,
                    from_pest_span(rule.as_span()),
                    TokenKind::Text {
                        is_whitespace: false,
                        content: rule.as_str().to_string(),
                    },
                ));
            }
        }
        tokens
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self.kind {
                TokenKind::Text { content, .. } => content.clone(),
                TokenKind::Identifier { name } => name.clone(),
                TokenKind::StringizeStart
                | TokenKind::StringizeEnd
                | TokenKind::Contactor
                | TokenKind::VaOptStart { .. }
                | TokenKind::VaOptEnd
                | TokenKind::CallStart { .. }
                | TokenKind::CallEnd { .. }
                | TokenKind::CallArgStart { .. }
                | TokenKind::CallArgEnd { .. }
                | TokenKind::EOI => "".to_string(),
                TokenKind::Newline => "\n".to_string(),
                TokenKind::Stringized(tokens) => format!(
                    "{:?}",
                    tokens.iter().map(|x| x.to_string()).collect::<String>()
                ),
                TokenKind::Contacted(left, right) => format!("{left}{right}"),
            }
        )
    }
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

pub fn to_string(tokens: &Vec<Token>) -> String {
    tokens.iter().map(|x| x.to_string()).collect::<String>()
}
