use crate::codegen::riscv::instruction::Instruction;

#[derive(Debug)]
pub struct BasicBlock {
    pub name: String,
    pub instructions: Vec<Instruction>,
    pub cursor: usize,
}

impl BasicBlock {
    pub fn new(name: &str) -> BasicBlock {
        BasicBlock {
            name: name.to_string(),
            instructions: vec![],
            cursor: 0,
        }
    }
}
