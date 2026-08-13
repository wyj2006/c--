use crate::codegen::riscv::{
    FP_REG,
    basic_block::InstructionId,
    function::Function,
    instruction::{Instruction, Opcode, Operand},
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
pub struct Reg {
    pub is_fp: bool,
    pub index: u64,
}

#[derive(Debug)]
pub struct ConflictGraph(BTreeMap<Reg, BTreeSet<Reg>>);

impl ConflictGraph {
    pub fn new() -> ConflictGraph {
        ConflictGraph(BTreeMap::new())
    }

    //添加一条无向边
    pub fn add_edge(&mut self, a: &Reg, b: &Reg) {
        if let None = self.0.get(a) {
            self.0.insert(*a, BTreeSet::new());
        }
        self.0.get_mut(a).unwrap().insert(*b);

        if let None = self.0.get(b) {
            self.0.insert(*b, BTreeSet::new());
        }
        self.0.get_mut(b).unwrap().insert(*a);
    }

    pub fn add_node(&mut self, a: &Reg) {
        if !self.0.contains_key(a) {
            self.0.insert(*a, BTreeSet::new());
        }
    }

    pub fn degree(&self, node: &Reg) -> usize {
        if let Some(t) = self.0.get(node) {
            t.len()
        } else {
            0
        }
    }

    pub fn get_nodes(&self) -> Vec<&Reg> {
        self.0.keys().collect()
    }

    pub fn neighbors(&self, node: &Reg) -> BTreeSet<Reg> {
        self.0.get(node).unwrap_or(&BTreeSet::new()).clone()
    }
}

#[derive(Debug)]
pub struct RegAllocator<'a> {
    pub function: &'a mut Function,
    pub live_in: BTreeMap<InstructionId, BTreeSet<Reg>>,
    pub live_out: BTreeMap<InstructionId, BTreeSet<Reg>>,
    pub defs: BTreeMap<InstructionId, BTreeSet<Reg>>,
    pub uses: BTreeMap<InstructionId, BTreeSet<Reg>>,
    pub conflict_graph: ConflictGraph,
    pub ireg_available: usize,
    pub freg_available: usize,
    pub assignment: BTreeMap<Reg, usize>,
    pub xlen: usize,
    //跟CodeGen中的ireg_num和freg_num功能类似
    pub reg_num: usize,
}

impl<'a> RegAllocator<'a> {
    pub fn new(function: &'a mut Function, xlen: usize) -> RegAllocator<'a> {
        RegAllocator {
            function,
            live_in: BTreeMap::new(),
            live_out: BTreeMap::new(),
            defs: BTreeMap::new(),
            uses: BTreeMap::new(),
            conflict_graph: ConflictGraph::new(),
            ireg_available: 7,
            freg_available: 12,
            assignment: BTreeMap::new(),
            xlen,
            reg_num: usize::MAX,
        }
    }

    pub fn allocate(&mut self) {
        loop {
            self.live_in.clear();
            self.live_out.clear();
            self.defs.clear();
            self.uses.clear();
            self.conflict_graph.0.clear();
            self.assignment.clear();

            self.build_defs_uses();
            self.liveness_analyze();
            self.build_conflict_graph();

            if self.assign() {
                break;
            }
        }
    }

    pub fn map_int_reg(index: u64) -> Operand {
        match index {
            0..=2 => Operand::IntReg(index + 5),
            3..=6 => Operand::IntReg(index - 3 + 28),
            _ => unreachable!(),
        }
    }

    pub fn map_fp_reg(index: u64) -> Operand {
        match index {
            0..=7 => Operand::FPReg(index),
            8..=11 => Operand::FPReg(index - 8 + 28),
            _ => unreachable!(),
        }
    }

    pub fn get_reg(&self, operand: &Operand) -> BTreeSet<Reg> {
        let mut set = BTreeSet::new();
        match operand {
            //小于32的情况是调用时用来传参的寄存器
            //但我要分配的是t0-6,ft0-11 不可能冲突
            Operand::IntReg(a) if *a >= 32 => {
                set.insert(Reg {
                    is_fp: false,
                    index: *a,
                });
            }
            Operand::FPReg(a) if *a >= 32 => {
                set.insert(Reg {
                    is_fp: true,
                    index: *a,
                });
            }
            Operand::Address { base, .. } => {
                set.extend(self.get_reg(base));
            }
            _ => {}
        }
        set
    }

    pub fn replace_reg(operand: &mut Operand, reg: &Reg, index: u64, is_virtual: bool) {
        match operand {
            Operand::IntReg(a) if !reg.is_fp && reg.index == *a => {
                *operand = if is_virtual {
                    Operand::IntReg(index)
                } else {
                    Self::map_int_reg(index)
                };
            }
            Operand::FPReg(a) if reg.is_fp && reg.index == *a => {
                *operand = if is_virtual {
                    Operand::FPReg(index)
                } else {
                    Self::map_fp_reg(index)
                };
            }
            Operand::Address { base, .. } => {
                Self::replace_reg(base, reg, index, is_virtual);
            }
            _ => {}
        }
    }

    pub fn build_defs_uses(&mut self) {
        for (_, basic_block) in self.function.basic_blocks.iter() {
            for (id, instruction) in basic_block.instructions.iter() {
                let mut defs = BTreeSet::new();
                let mut uses = BTreeSet::new();
                match instruction.opcode {
                    Opcode::Add
                    | Opcode::And
                    | Opcode::Div
                    | Opcode::DivU
                    | Opcode::FAddD
                    | Opcode::FAddS
                    | Opcode::FEqD
                    | Opcode::FEqS
                    | Opcode::FLeD
                    | Opcode::FLeS
                    | Opcode::FLtD
                    | Opcode::FLtS
                    | Opcode::FMulD
                    | Opcode::FMulS
                    | Opcode::FSubD
                    | Opcode::FSubS
                    | Opcode::LShift
                    | Opcode::Mul
                    | Opcode::Or
                    | Opcode::RShiftA
                    | Opcode::RShiftL
                    | Opcode::Rem
                    | Opcode::RemU
                    | Opcode::SetLt
                    | Opcode::SetLtU
                    | Opcode::Sub
                    | Opcode::Xor => {
                        defs.extend(self.get_reg(&instruction.operands[0]));
                        uses.extend(self.get_reg(&instruction.operands[1]));
                        uses.extend(self.get_reg(&instruction.operands[2]));
                    }
                    Opcode::BEq
                    | Opcode::FStoreD
                    | Opcode::FStoreS
                    | Opcode::StoreB
                    | Opcode::StoreD
                    | Opcode::StoreH
                    | Opcode::StoreW => {
                        uses.extend(self.get_reg(&instruction.operands[0]));
                        uses.extend(self.get_reg(&instruction.operands[1]));
                    }
                    Opcode::BEqZ | Opcode::BNeqZ => {
                        uses.extend(self.get_reg(&instruction.operands[0]));
                    }
                    Opcode::Call | Opcode::Jump | Opcode::Ret => {}
                    Opcode::FCvtDL
                    | Opcode::FCvtDLU
                    | Opcode::FCvtDS
                    | Opcode::FCvtDW
                    | Opcode::FCvtDWU
                    | Opcode::FCvtLD
                    | Opcode::FCvtLS
                    | Opcode::FCvtLUD
                    | Opcode::FCvtLUS
                    | Opcode::FCvtSD
                    | Opcode::FCvtSL
                    | Opcode::FCvtSLU
                    | Opcode::FCvtSW
                    | Opcode::FCvtSWU
                    | Opcode::FCvtWD
                    | Opcode::FCvtWS
                    | Opcode::FCvtWUD
                    | Opcode::FCvtWUS
                    | Opcode::FDivD
                    | Opcode::FDivS
                    | Opcode::FLoadD
                    | Opcode::FLoadS
                    | Opcode::FMoveD
                    | Opcode::FMoveDL
                    | Opcode::FMoveDW
                    | Opcode::FMoveLD
                    | Opcode::FMoveLS
                    | Opcode::FMoveS
                    | Opcode::FMoveSL
                    | Opcode::FMoveSW
                    | Opcode::FMoveWD
                    | Opcode::FMoveWS
                    | Opcode::FNegD
                    | Opcode::FNegS
                    | Opcode::LoadB
                    | Opcode::LoadBU
                    | Opcode::LoadD
                    | Opcode::LoadH
                    | Opcode::LoadHU
                    | Opcode::LoadW
                    | Opcode::LoadWU
                    | Opcode::Move
                    | Opcode::Neg
                    | Opcode::SetEqZ
                    | Opcode::SetGtZ
                    | Opcode::SetLtZ
                    | Opcode::SetNeqZ
                    | Opcode::BitNot => {
                        defs.extend(self.get_reg(&instruction.operands[0]));
                        uses.extend(self.get_reg(&instruction.operands[1]));
                    }
                    Opcode::LoadAddr | Opcode::LoadImm => {
                        defs.extend(self.get_reg(&instruction.operands[0]));
                    }
                }
                self.defs.insert(id.clone(), defs);
                self.uses.insert(id.clone(), uses);
            }
        }
    }

    pub fn liveness_analyze(&mut self) {
        let mut changed = true;
        while changed {
            changed = false;

            for (block_index, (_, basic_block)) in self.function.basic_blocks.iter().enumerate() {
                for (instr_index, (id, instruction)) in basic_block.instructions.iter().enumerate()
                {
                    let mut next_instr_ids = vec![];
                    let mut jumps = vec![];

                    //jmp/ret后的指令是不可达的
                    if !matches!(instruction.opcode, Opcode::Jump | Opcode::Ret) {
                        if let Some((id, _)) = basic_block.instructions.get_index(instr_index + 1) {
                            //下一条指令在当前基本块
                            next_instr_ids.push(id.clone());
                        } else {
                            //下一条指令在下一个基本块
                            if let Some((name, _)) =
                                self.function.basic_blocks.get_index(block_index + 1)
                            {
                                jumps.push(name.clone());
                            }
                        }
                    }

                    match instruction.opcode {
                        Opcode::BEq => match &instruction.operands[2] {
                            Operand::Symbol(name) => jumps.push(name.clone()),
                            _ => {}
                        },
                        Opcode::BEqZ | Opcode::BNeqZ => match &instruction.operands[1] {
                            Operand::Symbol(name) => jumps.push(name.clone()),
                            _ => {}
                        },
                        Opcode::Jump => match &instruction.operands[0] {
                            Operand::Symbol(name) => jumps.push(name.clone()),
                            _ => {}
                        },
                        _ => {}
                    }
                    for name in jumps {
                        let Some(mut index) = self.function.basic_blocks.get_index_of(&name) else {
                            continue;
                        };

                        loop {
                            let Some((_, basic_block)) =
                                self.function.basic_blocks.get_index(index)
                            else {
                                break;
                            };

                            if basic_block.instructions.len() == 0 {
                                index += 1;
                                continue;
                            }

                            next_instr_ids
                                .push(basic_block.instructions.get_index(0).unwrap().0.clone());
                            break;
                        }
                    }

                    let mut live_out = BTreeSet::new();
                    for next_instr_key in &next_instr_ids {
                        if let Some(t) = self.live_in.get(next_instr_key) {
                            live_out.extend(t);
                        }
                    }

                    let defs = if let Some(t) = self.defs.get(&id) {
                        t
                    } else {
                        &BTreeSet::new()
                    };
                    let uses = if let Some(t) = self.uses.get(&id) {
                        t
                    } else {
                        &BTreeSet::new()
                    };
                    let live_in = uses
                        .union(&live_out.difference(defs).copied().collect())
                        .copied()
                        .collect();

                    if let Some(t) = self.live_in.get_mut(&id) {
                        if *t != live_in {
                            changed = true;
                            *t = live_in;
                        }
                    } else {
                        changed = true;
                        self.live_in.insert(id.clone(), live_in);
                    }
                    if let Some(t) = self.live_out.get_mut(&id) {
                        if *t != live_out {
                            changed = true;
                            *t = live_out;
                        }
                    } else {
                        changed = true;
                        self.live_out.insert(id.clone(), live_out);
                    }
                }
            }
        }
    }

    pub fn build_conflict_graph(&mut self) {
        for (_, live_out) in &self.live_out {
            for a in live_out {
                for b in live_out {
                    if a == b || a.is_fp != b.is_fp {
                        continue;
                    }
                    self.conflict_graph.add_edge(a, b);
                }
            }
        }

        for (id, defs) in &self.defs {
            let Some(t) = self.live_out.get(id) else {
                continue;
            };

            for def in defs {
                for live_out in t {
                    if def == live_out || def.is_fp != live_out.is_fp {
                        continue;
                    }
                    self.conflict_graph.add_edge(def, live_out);
                }
            }
        }

        //添加孤立的节点
        for (_, regs) in &self.defs {
            for reg in regs {
                self.conflict_graph.add_node(reg);
            }
        }

        for (_, regs) in &self.uses {
            for reg in regs {
                self.conflict_graph.add_node(reg);
            }
        }
    }

    pub fn assign(&mut self) -> bool {
        let mut degree = BTreeMap::new();
        let mut stack = vec![];
        let mut removed = BTreeSet::new();
        let mut spilled = BTreeSet::new();

        for node in self.conflict_graph.get_nodes() {
            degree.insert(*node, self.conflict_graph.degree(node));
        }

        //强行溢出横跨调用的寄存器
        for (_, basic_block) in self.function.basic_blocks.iter() {
            for (id, instruction) in basic_block.instructions.iter() {
                if instruction.opcode == Opcode::Call {
                    let Some(live_out) = self.live_out.get(&id) else {
                        continue;
                    };
                    for reg in live_out {
                        //只处理虚拟寄存器
                        if reg.index >= 32 && !removed.contains(reg) {
                            removed.insert(*reg);
                            spilled.insert(*reg);

                            for neighbor in self.conflict_graph.neighbors(reg) {
                                degree.insert(neighbor, degree.get(&neighbor).unwrap() - 1);
                            }
                        }
                    }
                }
            }
        }

        //简化
        while removed.len() < degree.len() {
            let candiate = degree.iter().find(|(node, degree)| {
                !removed.contains(*node)
                //整数寄存器和浮点寄存器之间不可能有边
                    && ((!node.is_fp && **degree < self.ireg_available)
                        || (node.is_fp && **degree < self.freg_available))
            });

            match candiate {
                Some((node, _)) => {
                    removed.insert(*node);
                    stack.push(*node);

                    for neighbor in self.conflict_graph.neighbors(node) {
                        degree.insert(neighbor, degree.get(&neighbor).unwrap() - 1);
                    }
                }
                None => {
                    //溢出
                    //随便选一个
                    let candiate = degree.iter().find(|(node, _)| !removed.contains(*node));
                    match candiate {
                        Some((node, _)) => {
                            removed.insert(*node);
                            spilled.insert(*node);

                            for neighbor in self.conflict_graph.neighbors(node) {
                                degree.insert(neighbor, degree.get(&neighbor).unwrap() - 1);
                            }
                        }
                        None => break,
                    }
                }
            }
        }

        //着色
        while let Some(node) = stack.pop() {
            let mut used = vec![];

            for neighbor in self.conflict_graph.neighbors(&node) {
                if let Some(t) = self.assignment.get(&neighbor) {
                    used.push(*t);
                }
            }

            let reg = if node.is_fp {
                0..self.freg_available
            } else {
                0..self.ireg_available
            }
            .find(|x| !used.contains(x))
            .unwrap();
            self.assignment.insert(node, reg);
        }

        if spilled.len() == 0 {
            for (reg, index) in &self.assignment {
                for (_, basic_block) in self.function.basic_blocks.iter_mut() {
                    for (_, instruction) in basic_block.instructions.iter_mut() {
                        //替换成物理寄存器
                        for operand in instruction.operands.iter_mut() {
                            Self::replace_reg(operand, reg, *index as u64, false);
                        }
                    }
                }
            }
            return true;
        }

        for reg in spilled {
            if reg.is_fp {
                self.function.adjust_local_frame_size(8, 8);
            } else {
                self.function
                    .adjust_local_frame_size(self.xlen / 8, self.xlen / 8);
            }

            let frame_size = self.function.local_frame_size as i64;
            let spill_address = Operand::Address {
                base: Box::new(FP_REG),
                offset: -frame_size,
            };

            for (instr_id, regs) in &self.defs {
                if !regs.contains(&reg) {
                    continue;
                }
                //分配新的虚拟寄存器
                let reg_num = self.reg_num as u64;
                self.reg_num -= 1;

                let basic_block = self.function.basic_blocks.get_mut(&instr_id.0).unwrap();
                let instr_index = basic_block.instructions.get_index_of(instr_id).unwrap();
                let instruction = basic_block.instructions.get_mut(instr_id).unwrap();

                //替换溢出的寄存器
                match instruction.opcode {
                    Opcode::Add
                    | Opcode::And
                    | Opcode::Div
                    | Opcode::DivU
                    | Opcode::FAddD
                    | Opcode::FAddS
                    | Opcode::FEqD
                    | Opcode::FEqS
                    | Opcode::FLeD
                    | Opcode::FLeS
                    | Opcode::FLtD
                    | Opcode::FLtS
                    | Opcode::FMulD
                    | Opcode::FMulS
                    | Opcode::FSubD
                    | Opcode::FSubS
                    | Opcode::LShift
                    | Opcode::Mul
                    | Opcode::Or
                    | Opcode::RShiftA
                    | Opcode::RShiftL
                    | Opcode::Rem
                    | Opcode::RemU
                    | Opcode::SetLt
                    | Opcode::SetLtU
                    | Opcode::Sub
                    | Opcode::Xor
                    | Opcode::FCvtDL
                    | Opcode::FCvtDLU
                    | Opcode::FCvtDS
                    | Opcode::FCvtDW
                    | Opcode::FCvtDWU
                    | Opcode::FCvtLD
                    | Opcode::FCvtLS
                    | Opcode::FCvtLUD
                    | Opcode::FCvtLUS
                    | Opcode::FCvtSD
                    | Opcode::FCvtSL
                    | Opcode::FCvtSLU
                    | Opcode::FCvtSW
                    | Opcode::FCvtSWU
                    | Opcode::FCvtWD
                    | Opcode::FCvtWS
                    | Opcode::FCvtWUD
                    | Opcode::FCvtWUS
                    | Opcode::FDivD
                    | Opcode::FDivS
                    | Opcode::FLoadD
                    | Opcode::FLoadS
                    | Opcode::FMoveD
                    | Opcode::FMoveDL
                    | Opcode::FMoveDW
                    | Opcode::FMoveLD
                    | Opcode::FMoveLS
                    | Opcode::FMoveS
                    | Opcode::FMoveSL
                    | Opcode::FMoveSW
                    | Opcode::FMoveWD
                    | Opcode::FMoveWS
                    | Opcode::FNegD
                    | Opcode::FNegS
                    | Opcode::LoadB
                    | Opcode::LoadBU
                    | Opcode::LoadD
                    | Opcode::LoadH
                    | Opcode::LoadHU
                    | Opcode::LoadW
                    | Opcode::LoadWU
                    | Opcode::Move
                    | Opcode::Neg
                    | Opcode::SetEqZ
                    | Opcode::SetGtZ
                    | Opcode::SetLtZ
                    | Opcode::SetNeqZ
                    | Opcode::BitNot
                    | Opcode::LoadAddr
                    | Opcode::LoadImm => {
                        Self::replace_reg(&mut instruction.operands[0], &reg, reg_num, true);
                    }
                    _ => {}
                }

                //生成保存指令
                if reg.is_fp {
                    basic_block.cursor = instr_index + 1;
                    basic_block.add_instruction(Instruction::new(
                        Opcode::FStoreD,
                        &[Operand::FPReg(reg_num), spill_address.clone()],
                    ));
                } else {
                    basic_block.cursor = instr_index + 1;
                    basic_block.add_instruction(Instruction::new(
                        match self.xlen {
                            32 => Opcode::StoreW,
                            64 => Opcode::StoreD,
                            _ => unreachable!(),
                        },
                        &[Operand::IntReg(reg_num), spill_address.clone()],
                    ));
                }
            }

            for (instr_id, regs) in &self.uses {
                if !regs.contains(&reg) {
                    continue;
                }
                //分配新的虚拟寄存器
                let reg_num = self.reg_num as u64;
                self.reg_num -= 1;

                let basic_block = self.function.basic_blocks.get_mut(&instr_id.0).unwrap();
                let instr_index = basic_block.instructions.get_index_of(instr_id).unwrap();
                let instruction = basic_block.instructions.get_mut(instr_id).unwrap();

                //替换溢出的寄存器
                match instruction.opcode {
                    Opcode::Add
                    | Opcode::And
                    | Opcode::Div
                    | Opcode::DivU
                    | Opcode::FAddD
                    | Opcode::FAddS
                    | Opcode::FEqD
                    | Opcode::FEqS
                    | Opcode::FLeD
                    | Opcode::FLeS
                    | Opcode::FLtD
                    | Opcode::FLtS
                    | Opcode::FMulD
                    | Opcode::FMulS
                    | Opcode::FSubD
                    | Opcode::FSubS
                    | Opcode::LShift
                    | Opcode::Mul
                    | Opcode::Or
                    | Opcode::RShiftA
                    | Opcode::RShiftL
                    | Opcode::Rem
                    | Opcode::RemU
                    | Opcode::SetLt
                    | Opcode::SetLtU
                    | Opcode::Sub
                    | Opcode::Xor => {
                        Self::replace_reg(&mut instruction.operands[1], &reg, reg_num, true);
                        Self::replace_reg(&mut instruction.operands[2], &reg, reg_num, true);
                    }
                    Opcode::BEq
                    | Opcode::FStoreD
                    | Opcode::FStoreS
                    | Opcode::StoreB
                    | Opcode::StoreD
                    | Opcode::StoreH
                    | Opcode::StoreW => {
                        Self::replace_reg(&mut instruction.operands[0], &reg, reg_num, true);
                        Self::replace_reg(&mut instruction.operands[1], &reg, reg_num, true);
                    }
                    Opcode::BEqZ | Opcode::BNeqZ => {
                        Self::replace_reg(&mut instruction.operands[0], &reg, reg_num, true);
                    }
                    Opcode::Call | Opcode::Jump | Opcode::Ret => {}
                    Opcode::FCvtDL
                    | Opcode::FCvtDLU
                    | Opcode::FCvtDS
                    | Opcode::FCvtDW
                    | Opcode::FCvtDWU
                    | Opcode::FCvtLD
                    | Opcode::FCvtLS
                    | Opcode::FCvtLUD
                    | Opcode::FCvtLUS
                    | Opcode::FCvtSD
                    | Opcode::FCvtSL
                    | Opcode::FCvtSLU
                    | Opcode::FCvtSW
                    | Opcode::FCvtSWU
                    | Opcode::FCvtWD
                    | Opcode::FCvtWS
                    | Opcode::FCvtWUD
                    | Opcode::FCvtWUS
                    | Opcode::FDivD
                    | Opcode::FDivS
                    | Opcode::FLoadD
                    | Opcode::FLoadS
                    | Opcode::FMoveD
                    | Opcode::FMoveDL
                    | Opcode::FMoveDW
                    | Opcode::FMoveLD
                    | Opcode::FMoveLS
                    | Opcode::FMoveS
                    | Opcode::FMoveSL
                    | Opcode::FMoveSW
                    | Opcode::FMoveWD
                    | Opcode::FMoveWS
                    | Opcode::FNegD
                    | Opcode::FNegS
                    | Opcode::LoadB
                    | Opcode::LoadBU
                    | Opcode::LoadD
                    | Opcode::LoadH
                    | Opcode::LoadHU
                    | Opcode::LoadW
                    | Opcode::LoadWU
                    | Opcode::Move
                    | Opcode::Neg
                    | Opcode::SetEqZ
                    | Opcode::SetGtZ
                    | Opcode::SetLtZ
                    | Opcode::SetNeqZ
                    | Opcode::BitNot => {
                        Self::replace_reg(&mut instruction.operands[1], &reg, reg_num, true);
                    }
                    _ => {}
                }

                //生成加载指令
                if reg.is_fp {
                    basic_block.cursor = instr_index;
                    basic_block.add_instruction(Instruction::new(
                        Opcode::FLoadD,
                        &[Operand::FPReg(reg_num), spill_address.clone()],
                    ));
                } else {
                    basic_block.cursor = instr_index;
                    basic_block.add_instruction(Instruction::new(
                        match self.xlen {
                            32 => Opcode::LoadWU,
                            64 => Opcode::LoadD,
                            _ => unreachable!(),
                        },
                        &[Operand::IntReg(reg_num), spill_address.clone()],
                    ));
                }
            }
        }

        false
    }
}
