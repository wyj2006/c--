use crate::{
    codegen::riscv::{
        CodeGen,
        basic_block::BasicBlock,
        function::Function,
        instruction::{Instruction, Opcode, Operand},
    },
    ctype::{Type, layout::compute_layout},
    variant::Variant,
};
use num::ToPrimitive;
use std::{cell::RefCell, fmt::Display, rc::Rc};

impl CodeGen {
    pub fn display_variant(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        value: &Variant,
        r#type: &Rc<RefCell<Type>>,
    ) -> std::fmt::Result {
        match value {
            Variant::Bool(a) => writeln!(f, "    .byte {}", (*a) as u64)?,
            Variant::Int(a) => match r#type.borrow().size().unwrap() {
                1 => writeln!(f, "    .byte {}", a.to_i8().unwrap_or(i8::MAX))?,
                2 => writeln!(f, "    .half {}", a.to_i16().unwrap_or(i16::MAX))?,
                4 => writeln!(f, "    .word {}", a.to_i32().unwrap_or(i32::MAX))?,
                8 => writeln!(f, "    .dword {}", a.to_i64().unwrap_or(i64::MAX))?,
                _ => unreachable!(),
            },
            Variant::Nullptr => match self.xlen {
                32 => writeln!(f, "    .word 0")?,
                64 => writeln!(f, "    .dword 0")?,
                _ => unreachable!(),
            },
            Variant::Array(a) => {
                let layout = compute_layout(r#type.clone()).unwrap();
                for (i, v) in a.iter().enumerate() {
                    let child = &layout.children[i];
                    self.display_variant(f, v, &child.r#type)?;
                }
            }
            _ => unreachable!(),
        }
        Ok(())
    }
}

impl Display for CodeGen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (_, function) in &self.functions {
            if function.basic_blocks.len() > 0 {
                write!(f, "{}", function)?;
            }
        }

        for (name, (value, r#type)) in &self.globals {
            if r#type.borrow().is_function() {
                writeln!(f, "    .global {name}")?;
            } else {
                writeln!(f, "{name}:")?;
                match value {
                    Some(value) => self.display_variant(f, value, r#type)?,
                    None => writeln!(f, "    .zero {}", r#type.borrow().size().unwrap())?,
                }
            }
        }
        Ok(())
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}:", self.name)?;
        for (_, basic_block) in &self.basic_blocks {
            for i in format!("{basic_block}").split("\n") {
                if i.len() == 0 {
                    continue;
                }
                writeln!(f, "    {}", i)?;
            }
        }
        Ok(())
    }
}

impl Display for BasicBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}:", self.name)?;
        for (_, instruction) in &self.instructions {
            writeln!(f, "    {instruction}")?;
        }
        Ok(())
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}",
            self.opcode,
            self.operands
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Opcode::Add => "add",
                Opcode::And => "and",
                Opcode::BEq => "beq",
                Opcode::BEqZ => "beqz",
                Opcode::BNeqZ => "bnez",
                Opcode::BitNot => "not",
                Opcode::Call => "call",
                Opcode::Div => "div",
                Opcode::DivU => "divu",
                Opcode::FAddD => "fadd.d",
                Opcode::FAddS => "fadd.s",
                Opcode::FCvtDL => "fcvt.d.l",
                Opcode::FCvtDLU => "fcvt.d.lu",
                Opcode::FCvtDS => "fcvt.d.s",
                Opcode::FCvtDW => "fcvt.d.w",
                Opcode::FCvtDWU => "fcvt.d.wu",
                Opcode::FCvtLD => "fcvt.l.d",
                Opcode::FCvtLS => "fcvt.l.s",
                Opcode::FCvtLUD => "fcvt.lu.d",
                Opcode::FCvtLUS => "fcvt.lu.s",
                Opcode::FCvtSD => "fcvt.s.d",
                Opcode::FCvtSL => "fcvt.s.l",
                Opcode::FCvtSLU => "fcvt.s.lu",
                Opcode::FCvtSW => "fcvt.s.w",
                Opcode::FCvtSWU => "fcvt.s.wu",
                Opcode::FCvtWD => "fcvt.w.d",
                Opcode::FCvtWS => "fcvt.w.s",
                Opcode::FCvtWUD => "fcvt.wu.d",
                Opcode::FCvtWUS => "fcvt.wu.s",
                Opcode::FDivD => "fdiv.d",
                Opcode::FDivS => "fdiv.s",
                Opcode::FEqD => "feq.d",
                Opcode::FEqS => "feq.s",
                Opcode::FLeD => "fle.d",
                Opcode::FLeS => "fle.s",
                Opcode::FLoadD => "fld",
                Opcode::FLoadS => "flw",
                Opcode::FLtD => "flt.d",
                Opcode::FLtS => "flt.s",
                Opcode::FMoveD => "fmv.d",
                Opcode::FMoveDL => "fmv.d.x",
                Opcode::FMoveDW => "fmv.d.w",
                Opcode::FMoveLD => "fmv.x.d",
                Opcode::FMoveLS => "fmv.x.s",
                Opcode::FMoveS => "fmv.s",
                Opcode::FMoveSL => "fmv.s.x",
                Opcode::FMoveSW => "fmv.s.w",
                Opcode::FMoveWD => "fmv.w.d",
                Opcode::FMoveWS => "fmv.w.s",
                Opcode::FMulD => "fmul.d",
                Opcode::FMulS => "fmul.s",
                Opcode::FNegD => "fneg.d",
                Opcode::FNegS => "fneg.s",
                Opcode::FStoreD => "fsd",
                Opcode::FStoreS => "fsw",
                Opcode::FSubD => "fsub.d",
                Opcode::FSubS => "fsub.s",
                Opcode::Jump => "j",
                Opcode::LShift => "sll",
                Opcode::LoadAddr => "la",
                Opcode::LoadB => "lb",
                Opcode::LoadBU => "lbu",
                Opcode::LoadD => "ld",
                Opcode::LoadH => "lh",
                Opcode::LoadHU => "lhu",
                Opcode::LoadImm => "li",
                Opcode::LoadW => "lw",
                Opcode::LoadWU => "lwu",
                Opcode::Move => "mv",
                Opcode::Mul => "mul",
                Opcode::Neg => "neg",
                Opcode::Or => "or",
                Opcode::RShiftA => "sra",
                Opcode::RShiftL => "srl",
                Opcode::Rem => "rem",
                Opcode::RemU => "remu",
                Opcode::Ret => "ret",
                Opcode::SetEqZ => "seqz",
                Opcode::SetGtZ => "sgtz",
                Opcode::SetLt => "slt",
                Opcode::SetLtU => "sltu",
                Opcode::SetLtZ => "sltz",
                Opcode::SetNeqZ => "snez",
                Opcode::StoreB => "sb",
                Opcode::StoreD => "sd",
                Opcode::StoreH => "sh",
                Opcode::StoreW => "sw",
                Opcode::Sub => "sub",
                Opcode::Xor => "xor",
            }
        )
    }
}

impl Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Operand::Address { base, offset } => format!("{offset}({base})"),
                Operand::FPReg(a) => match a {
                    0..=7 => format!("ft{}", a),
                    10..=17 => format!("fa{}", a - 10),
                    28..=31 => format!("ft{}", a - 28 + 8),
                    _ => format!("f{a}"),
                },
                Operand::Immediate(a) => format!("{a}"),
                Operand::IntReg(a) => match a {
                    1 => "ra".to_string(),
                    2 => "sp".to_string(),
                    3 => "gp".to_string(),
                    4 => "tp".to_string(),
                    5..=7 => format!("t{}", a - 5),
                    8 => "fp".to_string(),
                    10..=17 => format!("a{}", a - 10),
                    28..=31 => format!("t{}", a - 28 + 3),
                    _ => format!("x{a}"),
                },
                Operand::Symbol(a) => format!("{a}"),
            }
        )
    }
}
