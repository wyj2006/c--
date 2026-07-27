use indexmap::IndexMap;

use crate::codegen::riscv::instruction::Instruction;

//.0是基本块的位置, .1是指令在块中的唯一id
pub type InstructionId = (String, usize);

#[derive(Debug)]
pub struct BasicBlock {
    pub name: String,
    pub instructions: IndexMap<InstructionId, Instruction>,
    pub cursor: usize,
}

impl BasicBlock {
    pub fn new(name: &str) -> BasicBlock {
        BasicBlock {
            name: name.to_string(),
            instructions: IndexMap::new(),
            cursor: 0,
        }
    }

    pub fn add_instruction(&mut self, instruction: Instruction) {
        let mut i = 0;
        while self.instructions.contains_key(&(self.name.clone(), i)) {
            i += 1;
        }

        self.instructions
            .insert_before(self.cursor, (self.name.clone(), i), instruction);
        self.cursor += 1;
    }
}
