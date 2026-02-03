use crate::preprocessor::tests::quick_new_preprocessor_with_name;

#[test]
pub fn include_direct() {
    let mut preprocessor = quick_new_preprocessor_with_name(
        "src/preprocessor/tests/include.string".to_string(),
        "#include \"include.txt\"\n",
    );
    let result = preprocessor.process().unwrap();
    assert_eq!(result, "\n\n\n\nchar p[] = \"x ## y\";\n\n");
}

#[test]
pub fn include_after_replace() {
    let mut preprocessor = quick_new_preprocessor_with_name(
        "src/preprocessor/tests/include.string".to_string(),
        "#define A \"include.txt\"
#include A
",
    );
    let result = preprocessor.process().unwrap();
    assert_eq!(result, "\n\n\n\n\nchar p[] = \"x ## y\";\n\n");
}

#[test]
pub fn include_use_macro() {
    let mut preprocessor = quick_new_preprocessor_with_name(
        "src/preprocessor/tests/include.string".to_string(),
        "#include \"include.txt\"\nchar p[] = join(x, y);\n",
    );
    let result = preprocessor.process().unwrap();
    assert_eq!(
        result,
        "\n\n\n\nchar p[] = \"x ## y\";\n\nchar p[] = \"x ## y\";\n"
    );
}
