use crate::files;
use codespan::Span;
use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    files::Files,
    term::{
        Config, emit_into_string, emit_to_write_style,
        termcolor::{ColorChoice, StandardStream},
    },
};
use inkwell::builder::BuilderError;
use pest::{RuleType, error::Error};

pub fn from_pest_span<'a>(span: pest::Span<'a>) -> Span {
    Span::new(span.start() as u32, span.end() as u32)
}

pub fn map_pest_err<T, R: RuleType>(
    file_id: usize,
    result: Result<T, Error<R>>,
) -> Result<T, Diagnostic<usize>> {
    match result {
        Ok(t) => Ok(t),
        Err(e) => Err(Diagnostic::error().with_message(format!(
            "\n{}",
            e.with_path(&files.lock().unwrap().name(file_id).unwrap())
        ))),
    }
}

pub fn map_builder_err<T>(
    file_id: usize,
    span: Span,
    result: Result<T, BuilderError>,
) -> Result<T, Diagnostic<usize>> {
    match result {
        Ok(t) => Ok(t),
        Err(e) => Err(Diagnostic::error()
            .with_message(format!("{e}"))
            .with_label(Label::primary(file_id, span))),
    }
}

pub fn warning(message: String, file_id: usize, span: Span, labels: Vec<Label<usize>>) {
    print_diag(
        Diagnostic::warning()
            .with_message(message)
            .with_label(Label::primary(file_id, span))
            .with_labels(labels),
    );
}

pub fn print_diag(diagnostic: Diagnostic<usize>) {
    let config = Config::default();
    if cfg!(not(test)) {
        let mut writer = StandardStream::stderr(ColorChoice::Always);
        emit_to_write_style(&mut writer, &config, &*files.lock().unwrap(), &diagnostic).unwrap();
    } else {
        println!(
            "{}",
            emit_into_string(&config, &*files.lock().unwrap(), &diagnostic).unwrap()
        );
    }
}
