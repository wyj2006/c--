mod ast;
mod ctype;
mod diagnostic;
mod parser;
mod preprocessor;
mod symtab;
mod typechecker;
mod variant;

use ast::printer::Print;
use parser::CParser;
use preprocessor::Preprocessor;
use std::{cell::RefCell, fs, rc::Rc};
use symtab::SymbolTable;
use typechecker::TypeChecker;

fn main() {
    let file_path = "test.txt".to_string();
    let file_content = fs::read_to_string(&file_path).unwrap();
    let mut preprocessor = Preprocessor::new(file_path.clone(), &file_content);
    let result = match preprocessor.process() {
        Ok(t) => t,
        Err(e) => {
            println!("error:\n{}", e);
            return;
        }
    };
    println!("{result}");

    let parser = CParser::new(&file_content);
    let ast = match parser.parse_to_ast() {
        Ok(t) => t,
        Err(e) => {
            println!("error:\n{}", e);
            return;
        }
    };

    let symtab = Rc::new(RefCell::new(SymbolTable::new()));
    let mut type_checker = TypeChecker::new(Rc::clone(&symtab));
    match type_checker.check(Rc::clone(&ast)) {
        Ok(t) => t,
        Err(e) => {
            ast.print();
            println!("error:\n{}", e);
            return;
        }
    }

    ast.print();
    println!();
    symtab.borrow().print(0);
}
