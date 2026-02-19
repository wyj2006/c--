pub mod bitfield;
pub mod init;

#[macro_export]
macro_rules! codegen_test_template {
    ($name:ident,$code:expr) => {
        #[ignore = "bug in inkwell"]
        #[test]
        pub fn $name() {
            use crate::codegen::CodeGen;
            use crate::{
                symtab::SymbolTable,
                typechecker::{TypeChecker, tests::quick_new_parser},
            };
            use inkwell::{
                context::Context,
                targets::{InitializationConfig, Target, TargetMachine},
            };
            use insta::assert_snapshot;
            use std::{cell::RefCell, rc::Rc};

            let parser = quick_new_parser($code);
            let ast = parser.parse_to_ast().unwrap();

            let symtab = Rc::new(RefCell::new(SymbolTable::new()));
            let mut type_checker = TypeChecker::new(Rc::clone(&symtab));
            type_checker.check(Rc::clone(&ast)).unwrap();

            Target::initialize_all(&InitializationConfig::default());
            let target_triple = TargetMachine::get_default_triple();

            let context = Context::create();
            let module = context.create_module("<string>");
            module.set_triple(&target_triple);

            let mut codegen = CodeGen::new(&context, module);
            codegen.r#gen(Rc::clone(&ast)).unwrap();

            assert_snapshot!(codegen.module.to_string());
        }
    };
}
