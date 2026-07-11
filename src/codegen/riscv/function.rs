use crate::codegen::riscv::{basic_block::BasicBlock, instruction::Operand};
use indexmap::IndexMap;

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub basic_blocks: IndexMap<String, BasicBlock>,
    pub frame_size: usize,
    pub arg_frame_size: usize, //跟frame_size, 用来表示接收参数
    pub cursor: usize,
    //参数中使用整数和浮点寄存器的数量
    pub ireg_used: u64,
    pub freg_used: u64,
    pub ra_saved: Operand, //ra寄存器保存的地址
    pub a0_saved: Operand,
}

impl Function {
    pub fn new(name: &str) -> Function {
        Function {
            name: name.to_string(),
            basic_blocks: IndexMap::new(),
            frame_size: 0,
            arg_frame_size: 0,
            cursor: usize::MAX,
            ireg_used: 0,
            freg_used: 0,
            ra_saved: Operand::Immediate(0),
            a0_saved: Operand::Immediate(0),
        }
    }
}
