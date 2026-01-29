pub mod decl;
pub mod expr;
pub mod init;

#[macro_export]
macro_rules! test_template {
    ($name:ident,$code:expr) => {
        #[test]
        pub fn $name() {
            let parser = CParser::new($code);
            let ast = parser.parse_to_ast().unwrap();

            let symtab = Rc::new(RefCell::new(SymbolTable::new()));
            let mut type_checker = TypeChecker::new(Rc::clone(&symtab));
            type_checker.check(Rc::clone(&ast)).unwrap();

            assert_debug_snapshot!(ast);
        }
    };
}
