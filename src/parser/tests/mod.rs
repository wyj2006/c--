pub mod ambiguity;
pub mod expr;

#[macro_export]
macro_rules! parser_test_template {
    ($name:ident,$code:expr) => {
        #[test]
        pub fn $name() {
            let parser = CParser::new($code);
            let ast = parser.parse_to_ast().unwrap();

            assert_debug_snapshot!(ast);
        }
    };
}
