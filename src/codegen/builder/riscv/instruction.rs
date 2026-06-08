use crate::codegen::builder::riscv::value::RISCVValue;
use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Clone)]
pub struct Instruction {
    pub opcode: String,
    pub operands: Vec<Rc<RefCell<RISCVValue>>>,
}
