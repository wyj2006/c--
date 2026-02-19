pub mod ast;
pub mod codegen;
pub mod ctype;
pub mod diagnostic;
pub mod file_map;
pub mod parser;
pub mod preprocessor;
pub mod symtab;
pub mod typechecker;
pub mod variant;

use crate::{
    codegen::CodeGen,
    diagnostic::print_diag,
    file_map::{FileMap, source_map},
};
use ast::printer::Print;
use codespan_reporting::diagnostic::Diagnostic;
use inkwell::{
    OptimizationLevel,
    context::Context,
    targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine},
};
use lazy_static::lazy_static;
use parser::CParser;
use preprocessor::Preprocessor;
use std::{cell::RefCell, fs, path::Path, rc::Rc, sync::Mutex};
use symtab::SymbolTable;
use typechecker::TypeChecker;

lazy_static! {
    pub static ref files: Mutex<FileMap> = Mutex::new(FileMap::new());
}

fn main() {
    let file_path = "test.txt";
    let file_content = fs::read_to_string(&file_path).unwrap();

    let source_id = files
        .lock()
        .unwrap()
        .add(file_path.to_string(), file_content.to_string());

    let mut preprocessor = Preprocessor::new(source_id);
    let tokens = match preprocessor.process() {
        Ok(t) => t,
        Err(e) => {
            print_diag(e);
            return;
        }
    };

    let parser = CParser::new(source_map(file_path.to_string(), &tokens));
    let ast = match parser.parse_to_ast() {
        Ok(t) => t,
        Err(e) => {
            print_diag(e);
            return;
        }
    };

    let symtab = Rc::new(RefCell::new(SymbolTable::new()));
    ast.borrow_mut().symtab = Some(Rc::clone(&symtab));
    let mut type_checker = TypeChecker::new(Rc::clone(&symtab));
    match type_checker.check(Rc::clone(&ast)) {
        Ok(t) => t,
        Err(e) => {
            ast.print();
            println!();
            symtab.borrow().print(0);
            println!();
            print_diag(e);
            return;
        }
    }

    ast.print();
    println!();
    symtab.borrow().print(0);

    Target::initialize_all(&InitializationConfig::default());
    let target_triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&target_triple).unwrap();
    let target_machine = target
        .create_target_machine(
            &target_triple,
            "generic",
            "",
            OptimizationLevel::Default,
            RelocMode::Default,
            CodeModel::Default,
        )
        .unwrap();

    let context = Context::create();
    let module = context.create_module(file_path);
    module.set_triple(&target_triple);

    let mut codegen = CodeGen::new(&context, module);
    match codegen.r#gen(Rc::clone(&ast)) {
        Ok(t) => t,
        Err(e) => {
            print_diag(e);
            return;
        }
    }

    println!();
    let module_string = codegen.module.print_to_string();
    println!("{}", module_string.to_string());

    let mut path = Path::new(file_path).to_path_buf();
    path.set_extension("o");
    match target_machine.write_to_file(&codegen.module, FileType::Object, &path) {
        Ok(_) => {}
        Err(e) => {
            print_diag(
                Diagnostic::error().with_message(format!("cannot generate object file: {e}")),
            );
            return;
        }
    }
}
