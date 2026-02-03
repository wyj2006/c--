pub mod expr;

#[macro_export]
macro_rules! parser_test_template {
    ($name:ident,$code:expr) => {
        #[test]
        pub fn $name() {
            use crate::files;
            use crate::parser::CParser;
            use insta::assert_snapshot;
            use regex::Regex;

            let source_id = files
                .lock()
                .unwrap()
                .add("<string>".to_string(), $code.to_string());

            let parser = CParser::new(source_id);
            let ast = parser.parse_to_ast().unwrap();

            assert_snapshot!(
                Regex::new(r#"define_loc[\s\S]*?name"#)
                    .unwrap()
                    .replace_all(
                        &Regex::new(r#"\n.*?file_id[\s\S]*?,"#)
                            .unwrap()
                            .replace_all(format!("{ast:#?}").as_str(), ""),
                        "name"
                    )
            );
        }
    };
}
