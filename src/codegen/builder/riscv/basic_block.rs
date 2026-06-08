use crate::codegen::builder::riscv::instruction::Instruction;

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub name: String,
    pub instructions: Vec<Instruction>,
    pub cursor: usize, //下一个指令的插入位置
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
