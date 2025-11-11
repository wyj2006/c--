use crate::*;

#[test]
pub fn include_direct() {
    let mut preprocessor = Preprocessor::new(
        "src/preprocessor/tests/include.string".to_string(),
        "#include \"include.txt\"
"
        .to_string(),
    );
    let result = preprocessor.process().unwrap();
    assert_eq!(result, "\n\n\n\nchar p[] = \"x ## y\";\n\n");
}

#[test]
pub fn include_after_replace() {
    let mut preprocessor = Preprocessor::new(
        "src/preprocessor/tests/include.string".to_string(),
        "#define A \"include.txt\"
#include A
"
        .to_string(),
    );
    let result = preprocessor.process().unwrap();
    assert_eq!(result, "\n\n\n\n\nchar p[] = \"x ## y\";\n\n");
}
