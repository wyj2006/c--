use crate::codegen::riscv::basic_block::BasicBlock;
use indexmap::IndexMap;

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub basic_blocks: IndexMap<String, BasicBlock>,
    pub frame_size: usize,
    pub cursor: usize,
    //参数中使用整数和浮点寄存器的数量
    pub ireg_used: u64,
    pub freg_used: u64,
}

impl Function {
    pub fn new(name: &str) -> Function {
        Function {
            name: name.to_string(),
            basic_blocks: IndexMap::new(),
            frame_size: 0,
            cursor: usize::MAX,
            ireg_used: 0,
            freg_used: 0,
        }
    }
}
