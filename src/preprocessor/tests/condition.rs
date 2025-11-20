use crate::*;

#[test]
pub fn condition_without_if() {
    let mut preprocessor = Preprocessor::new(
        "<string>".to_string(),
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
    let mut preprocessor = Preprocessor::new(
        "<string>".to_string(),
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
    let mut preprocessor = Preprocessor::new(
        "src/preprocessor/tests/condition.string".to_string(),
        "#if !defined(A)
#if __has_embed( \"embed.txt\" limit(1?1+1:1))
2
#else
3
#endif
#endif
",
    );
    let result = preprocessor.process().unwrap();
    assert_eq!(result, "\n\n2\n");
}
