use crate::preprocessor::{tests::quick_new_preprocessor_with_name, token::to_string};

#[test]
pub fn embed_without_parameter() {
    let mut preprocessor = quick_new_preprocessor_with_name(
        "src/preprocessor/tests/embed.string".to_string(),
        "#embed \"embed.txt\"\n",
    );
    let result = to_string(&preprocessor.process().unwrap());
    assert_eq!(result, "48, 49, 50, 51\n");
}

#[test]
pub fn embed_with_limit() {
    let mut preprocessor = quick_new_preprocessor_with_name(
        "src/preprocessor/tests/embed.string".to_string(),
        "#embed \"embed.txt\" limit(1?1+1:1)\n",
    );
    let result = to_string(&preprocessor.process().unwrap());
    assert_eq!(result, "48, 49\n");
}

#[test]
pub fn embed_with_prefix() {
    let mut preprocessor = quick_new_preprocessor_with_name(
        "src/preprocessor/tests/embed.string".to_string(),
        "#embed \"embed.txt\" prefix(abc )\n",
    );
    let result = to_string(&preprocessor.process().unwrap());
    assert_eq!(result, "abc 48, 49, 50, 51\n");
}

#[test]
pub fn embed_with_suffix() {
    let mut preprocessor = quick_new_preprocessor_with_name(
        "src/preprocessor/tests/embed.string".to_string(),
        "#embed \"embed.txt\" suffix( ,abc)\n",
    );
    let result = to_string(&preprocessor.process().unwrap());
    assert_eq!(result, "48, 49, 50, 51 ,abc\n");
}

#[test]
pub fn embed_with_if_empty() {
    let mut preprocessor = quick_new_preprocessor_with_name(
        "src/preprocessor/tests/embed.string".to_string(),
        "#embed \"embed.txt\" if_empty(0)\n",
    );
    let result = to_string(&preprocessor.process().unwrap());
    assert_eq!(result, "48, 49, 50, 51\n");
}
