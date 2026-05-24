pub mod decl;
pub mod expr;
pub mod init;
pub mod stmt;

#[macro_export]
macro_rules! codegen_test_template {
    ($name:ident,$code:expr) => {
        #[cfg(not(windows))]
        #[test]
        pub fn $name() {
            use crate::codegen::{CodeGen, builder::llvmir::LLVMIRBuilder};
            use crate::{
                symtab::SymbolTable,
                typechecker::{TypeChecker, tests::quick_new_parser},
            };
            use inkwell::{
                context::Context,
                targets::{InitializationConfig, Target, TargetTriple},
            };
            use insta::assert_snapshot;
            use std::{cell::RefCell, rc::Rc};

            let parser = quick_new_parser($code);
            let ast = parser.parse_to_ast().unwrap();

            let symtab = Rc::new(RefCell::new(SymbolTable::new()));
            ast.borrow_mut().symtab = Some(Rc::clone(&symtab));
            let mut type_checker = TypeChecker::new(Rc::clone(&symtab));
            type_checker.check(Rc::clone(&ast)).unwrap();

            Target::initialize_all(&InitializationConfig::default());
            let target_triple = TargetTriple::create("x86_64-pc-linux-gnu");

            let context = Context::create();
            let module = context.create_module("<string>");
            module.set_triple(&target_triple);

            let builder = LLVMIRBuilder::new(&context, module);

            let mut codegen = CodeGen::new(builder);
            codegen.r#gen(Rc::clone(&ast)).unwrap();

            assert_snapshot!(codegen.builder.module.to_string());
        }
    };
}
