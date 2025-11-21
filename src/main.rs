mod ast;
mod diagnostic;
mod parser;
mod preprocessor;

use parser::CParser;
use parser::Rule;
use pest::Parser;
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
    match CParser::parse(Rule::translation_unit, result.as_str()) {
        Ok(t) => println!("{}", t.to_json()),
        Err(e) => println!("{e}"),
    }
}
