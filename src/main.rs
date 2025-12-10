mod ast;
mod ctype;
mod diagnostic;
mod parser;
mod preprocessor;

use ast::printer::Print;
use parser::CParser;
use preprocessor::Preprocessor;
use std::fs;

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
    ast.print();
}
