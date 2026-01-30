use crate::parser;
use pest::Span;
use pest::error::{Error, ErrorVariant};

#[derive(Debug)]
pub struct Diagnostic<'a> {
    pub span: Span<'a>,
    pub kind: DiagnosticKind,
    pub message: String,
    pub notes: Vec<Diagnostic<'a>>,
}

#[derive(Debug)]
pub enum DiagnosticKind {
    Error,
    Warning,
    Note,
}

impl<'a> Diagnostic<'a> {
    pub fn print(&self) {
        self.print_with_path("");
    }

    pub fn print_with_path(&self, path: &str) {
        for note in &self.notes {
            note.print();
        }
        match self.kind {
            DiagnosticKind::Error => println!("error:"),
            DiagnosticKind::Warning => println!("warning:"),
            DiagnosticKind::Note => println!("note:"),
        }
        println!(
            "{}",
            Error::<parser::Rule>::new_from_span(
                ErrorVariant::CustomError {
                    message: self.message.clone()
                },
                self.span
            )
            .with_path(path)
        );
    }
}

pub fn warning(message: String, span: Span, path: &str) {
    Diagnostic {
        span,
        kind: DiagnosticKind::Warning,
        message,
        notes: vec![],
    }
    .print_with_path(path);
}
