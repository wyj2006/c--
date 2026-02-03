pub mod decl;
pub mod expr;
pub mod init;

use crate::{files, parser::CParser};

#[macro_export]
macro_rules! typechecker_test_template {
    ($name:ident,$code:expr) => {
        #[test]
        pub fn $name() {
            use insta::assert_snapshot;
            use regex::Regex;

            let parser = quick_new_parser($code);
            let ast = parser.parse_to_ast().unwrap();

            let symtab = Rc::new(RefCell::new(SymbolTable::new()));
            let mut type_checker = TypeChecker::new(Rc::clone(&symtab));
            type_checker.check(Rc::clone(&ast)).unwrap();

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

pub fn quick_new_parser<T: ToString>(code: T) -> CParser {
    let source_id = files
        .lock()
        .unwrap()
        .add("<string>".to_string(), code.to_string());

    CParser::new(source_id)
}
