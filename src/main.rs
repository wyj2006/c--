pub mod ast;
pub mod ctype;
pub mod diagnostic;
pub mod file_map;
pub mod parser;
pub mod preprocessor;
pub mod symtab;
pub mod typechecker;
pub mod variant;

use crate::{diagnostic::print_diag, file_map::FileMap};
use ast::printer::Print;
use lazy_static::lazy_static;
use parser::CParser;
use preprocessor::Preprocessor;
use std::{cell::RefCell, fs, rc::Rc, sync::Mutex};
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
    let result = match preprocessor.process() {
        Ok(t) => t,
        Err(e) => {
            print_diag(e);
            return;
        }
    };
    println!("{result}");

    let pp_id = files.lock().unwrap().add(file_path.to_string(), result);

    let parser = CParser::new(pp_id);
    let ast = match parser.parse_to_ast() {
        Ok(t) => t,
        Err(e) => {
            print_diag(e);
            return;
        }
    };

    let symtab = Rc::new(RefCell::new(SymbolTable::new()));
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
}
