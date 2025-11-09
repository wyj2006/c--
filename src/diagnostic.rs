use pest::error::{Error, ErrorVariant};
use pest::{RuleType, Span};

pub fn warning<T: RuleType>(message: String, span: Span, path: &str) {
    println!(
        "warning:\n{}",
        Error::<T>::new_from_span(ErrorVariant::CustomError { message }, span).with_path(path)
    );
}
