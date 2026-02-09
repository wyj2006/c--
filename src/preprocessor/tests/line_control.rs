use crate::preprocessor::{tests::quick_new_preprocessor, token::to_string};

#[test]
pub fn line_control() {
    let mut preprocessor = quick_new_preprocessor(
        "#line 3
__LINE__
__LINE__
",
    );
    let result = to_string(&preprocessor.process().unwrap());
    assert_eq!(result, "\n3\n4\n");
}

#[test]
pub fn line_control_with_file() {
    let mut preprocessor = quick_new_preprocessor(
        "#line 3 \"string2\"
__LINE__ __FILE__
",
    );
    let result = to_string(&preprocessor.process().unwrap());
    assert_eq!(result, "\n3 \"string2\"\n");
}
