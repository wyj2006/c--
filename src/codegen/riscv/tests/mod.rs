pub mod decl;
pub mod expr;
pub mod init;
pub mod stmt;

#[macro_export]
macro_rules! codegen_riscv_test_template {
    ($name:ident,$code:expr) => {
        #[test]
        pub fn $name() {
            use crate::codegen::riscv::CodeGen;
            use crate::{
                symtab::SymbolTable,
                typechecker::{TypeChecker, tests::quick_new_parser},
            };
            use insta::assert_snapshot;
            use std::{cell::RefCell, rc::Rc};

            let parser = quick_new_parser($code);
            let ast = parser.parse_to_ast().unwrap();

            let symtab = Rc::new(RefCell::new(SymbolTable::new()));
            ast.borrow_mut().symtab = Some(Rc::clone(&symtab));
            let mut type_checker = TypeChecker::new(Rc::clone(&symtab));
            type_checker.check(Rc::clone(&ast)).unwrap();

            let mut codegen = CodeGen::new();
            codegen.r#gen(&ast).unwrap();

            assert_snapshot!(codegen.to_string());
        }
    };
}
