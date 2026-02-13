pub mod expr;

#[macro_export]
macro_rules! parser_test_template {
    ($name:ident,$code:expr) => {
        #[test]
        pub fn $name() {
            use crate::ast::printer::Print;
            use crate::files;
            use crate::parser::CParser;
            use insta::assert_snapshot;

            let source_id = files
                .lock()
                .unwrap()
                .add("<string>".to_string(), $code.to_string());

            let parser = CParser::new(source_id);
            let ast = parser.parse_to_ast().unwrap();

            assert_snapshot!(ast.print_line(0).join("\n"));
        }
    };
}
