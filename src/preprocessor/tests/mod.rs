use crate::{files, preprocessor::Preprocessor};

pub mod condition;
pub mod embed;
pub mod include;
pub mod line_control;
pub mod macro_related;

pub fn quick_new_preprocessor<T: ToString>(code: T) -> Preprocessor {
    quick_new_preprocessor_with_name("<string>", code)
}

pub fn quick_new_preprocessor_with_name<T: ToString, E: ToString>(
    name: T,
    code: E,
) -> Preprocessor {
    let source_id = files
        .lock()
        .unwrap()
        .add(name.to_string(), code.to_string());
    Preprocessor::new(source_id)
}
