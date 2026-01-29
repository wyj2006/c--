use pest::{
    Span,
    error::{self, ErrorVariant},
};
use std::fmt::Display;

use crate::parser;

#[derive(Debug)]
pub struct Error<'a> {
    pub span: Span<'a>,
    pub kind: ErrorKind,
}

#[derive(Debug)]
pub enum ErrorKind {
    Redefine(String),
    Undefine(String),
    UnexpectStorageClass,
    TooManyStorageClass,
    TooLargeChar,
}

impl Display for Error<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error::Error::<parser::Rule>::new_from_span(
            ErrorVariant::CustomError {
                message: match &self.kind {
                    ErrorKind::Redefine(name) => format!("Redefine {name}"),
                    ErrorKind::Undefine(name) => format!("Undefine {name}"),
                    ErrorKind::UnexpectStorageClass => format!("Unexpect storage class"),
                    ErrorKind::TooManyStorageClass => format!("Too many storage class"),
                    ErrorKind::TooLargeChar => format!("Character too large"),
                },
            },
            self.span,
        )
        .fmt(f)
    }
}
