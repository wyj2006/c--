use crate::preprocessor::tests::{quick_new_preprocessor, quick_new_preprocessor_with_name};

#[test]
pub fn condition_without_if() {
    let mut preprocessor = quick_new_preprocessor(
        "#ifdef A
1
#elifdef B
2
#else
3
#endif
",
    );
    let result = preprocessor.process().unwrap();
    assert_eq!(result, "\n3\n");
}

#[test]
pub fn condition_with_if() {
    let mut preprocessor = quick_new_preprocessor(
        "#if defined(A)
1
#elif !defined(B)
2
#else
3
#endif
",
    );
    let result = preprocessor.process().unwrap();
    assert_eq!(result, "\n2\n");
}

#[test]
pub fn nested_condition() {
    let mut preprocessor = quick_new_preprocessor_with_name(
        "src/preprocessor/tests/condition.string".to_string(),
        r#"#if !defined(A)
#if __has_embed( "embed.txt" limit(1?1+1:1))
2
#else
3
#endif
#endif
"#
        .to_string(),
    );
    let result = preprocessor.process().unwrap();
    assert_eq!(result, "\n\n2\n");
}
