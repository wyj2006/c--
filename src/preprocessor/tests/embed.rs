use crate::*;

#[test]
pub fn embed_without_parameter() {
    let mut preprocessor = Preprocessor::new(
        "src/preprocessor/tests/embed.string".to_string(),
        "#embed \"embed.txt\"\n".to_string(),
    );
    let result = preprocessor.process().unwrap();
    assert_eq!(result, "48,49,50,51\n");
}

#[test]
pub fn embed_with_limit() {
    let mut preprocessor = Preprocessor::new(
        "src/preprocessor/tests/embed.string".to_string(),
        "#embed \"embed.txt\" limit(1?1+1:1)\n".to_string(),
    );
    let result = preprocessor.process().unwrap();
    assert_eq!(result, "48,49\n");
}

#[test]
pub fn embed_with_prefix() {
    let mut preprocessor = Preprocessor::new(
        "src/preprocessor/tests/embed.string".to_string(),
        "#embed \"embed.txt\" prefix(abc )\n".to_string(),
    );
    let result = preprocessor.process().unwrap();
    assert_eq!(result, "abc 48,49,50,51\n");
}

#[test]
pub fn embed_with_suffix() {
    let mut preprocessor = Preprocessor::new(
        "src/preprocessor/tests/embed.string".to_string(),
        "#embed \"embed.txt\" suffix( ,abc)\n".to_string(),
    );
    let result = preprocessor.process().unwrap();
    assert_eq!(result, "48,49,50,51 ,abc\n");
}

#[test]
pub fn embed_with_if_empty() {
    let mut preprocessor = Preprocessor::new(
        "src/preprocessor/tests/embed.string".to_string(),
        "#embed \"embed.txt\" if_empty(0)\n".to_string(),
    );
    let result = preprocessor.process().unwrap();
    assert_eq!(result, "48,49,50,51\n");
}
