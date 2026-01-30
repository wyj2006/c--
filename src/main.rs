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
    let file_path = "test.txt";
    let file_content = fs::read_to_string(&file_path).unwrap();
    let mut preprocessor = Preprocessor::new(file_path.to_string(), &file_content);
    let result = match preprocessor.process() {
        Ok(t) => t,
        Err(e) => {
            println!("error:\n{}", e.with_path(&file_path));
            return;
        }
    };
    println!("{result}");

    let parser = CParser::new(&file_content);
    let ast = match parser.parse_to_ast() {
        Ok(t) => t,
        Err(e) => {
            println!("error:\n{}", e.with_path(&file_path));
            return;
        }
    };

    let symtab = Rc::new(RefCell::new(SymbolTable::new()));
    let mut type_checker = TypeChecker::new(file_path, Rc::clone(&symtab));
    match type_checker.check(Rc::clone(&ast)) {
        Ok(t) => t,
        Err(e) => {
            ast.print();
            println!();
            symtab.borrow().print(0);
            println!();
            e.print_with_path(&file_path);
            return;
        }
    }

    ast.print();
    println!();
    symtab.borrow().print(0);
}
