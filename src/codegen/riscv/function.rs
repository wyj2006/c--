use crate::codegen::riscv::basic_block::BasicBlock;
use indexmap::IndexMap;

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub basic_blocks: IndexMap<String, BasicBlock>,
    pub local_frame_size: usize, //局部变量占用的栈大小
    pub param_frame_size: usize, //跟frame_size, 用来表示接收参数
    pub arg_frame_size: usize,   //函数体内的函数调用使用的最大栈大小
    pub cursor: usize,
    //参数中使用整数和浮点寄存器的数量
    pub ireg_used: u64,
    pub freg_used: u64,
    pub ireg_saved: IndexMap<usize, i64>,
}

impl Function {
    pub fn new(name: &str) -> Function {
        Function {
            name: name.to_string(),
            basic_blocks: IndexMap::new(),
            local_frame_size: 0,
            param_frame_size: 0,
            arg_frame_size: 0,
            cursor: usize::MAX,
            ireg_used: 0,
            freg_used: 0,
            ireg_saved: IndexMap::new(),
        }
    }
}
