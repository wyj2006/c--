pub mod decl;
pub mod expr;
pub mod init;
pub mod stmt;

use crate::{files, parser::CParser};

#[macro_export]
macro_rules! typechecker_test_template {
    ($name:ident,$code:expr) => {
        #[test]
        pub fn $name() {
            use crate::ast::printer::Print;
            use crate::{
                symtab::SymbolTable,
                typechecker::{TypeChecker, tests::quick_new_parser},
            };
            use insta::assert_snapshot;
            use std::{cell::RefCell, rc::Rc};

            let parser = quick_new_parser($code);
            let ast = parser.parse_to_ast().unwrap();

            let symtab = Rc::new(RefCell::new(SymbolTable::new()));
            let mut type_checker = TypeChecker::new(Rc::clone(&symtab));
            type_checker.check(Rc::clone(&ast)).unwrap();

            assert_snapshot!(ast.print_line(0).join("\n"));
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
