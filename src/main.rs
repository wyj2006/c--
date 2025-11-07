mod preprocessor;

use preprocessor::Preprocessor;
use std::fs;

fn main() {
    let file_path = "test.txt".to_string();
    let mut preprocessor =
        Preprocessor::new(file_path.clone(), fs::read_to_string(&file_path).unwrap());
    let result = preprocessor.process().unwrap();
    println!("{result}");
}
