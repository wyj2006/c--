use crate::*;

#[test]
pub fn line_control() {
    let mut preprocessor = Preprocessor::new(
        "<string>".to_string(),
        "#line 3
__LINE__
__LINE__
",
    );
    let result = preprocessor.process().unwrap();
    assert_eq!(result, "\n3\n4\n");
}

#[test]
pub fn line_control_with_file() {
    let mut preprocessor = Preprocessor::new(
        "<string>".to_string(),
        "#line 3 \"string2\"
__LINE__ __FILE__
",
    );
    let result = preprocessor.process().unwrap();
    assert_eq!(result, "\n3 \"string2\"\n");
}
