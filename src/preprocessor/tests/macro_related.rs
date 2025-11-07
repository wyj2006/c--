use crate::*;

#[test]
pub fn macro_define() {
    let mut preprocessor = Preprocessor::new("<string>".to_string(), "#define PI\n".to_string());
    let result = preprocessor.process().unwrap();
    assert_eq!(result, "\n");
}
