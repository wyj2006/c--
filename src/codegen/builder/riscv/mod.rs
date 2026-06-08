pub mod basic_block;
pub mod builder_impl;
pub mod instruction;
pub mod value;

use crate::{
    ast::expr::BinOpKind,
    codegen::builder::{
        Builder,
        riscv::{basic_block::BasicBlock, instruction::Instruction, value::RISCVValue},
    },
    ctype::{Type, TypeKind},
    symtab::SymbolTable,
};
use codespan::Span;
use codespan_reporting::diagnostic::Diagnostic;
use indexmap::IndexMap;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub struct RISCVBuilder {
    pub file_ids: Vec<usize>,
    pub spans: Vec<Span>,
    pub symtabs: Vec<Rc<RefCell<SymbolTable>>>,
    pub globals: IndexMap<String, Option<Rc<RefCell<RISCVValue>>>>,
    pub text_section: IndexMap<String, Rc<RefCell<BasicBlock>>>,
    pub cur_block_name: String,
    pub func_values: Vec<Rc<RefCell<RISCVValue>>>,
    pub values: HashMap<usize, Rc<RefCell<RISCVValue>>>,
    pub ireg_num: usize,
    pub freg_num: usize,
    pub xlen: usize,
}

impl RISCVBuilder {
    pub fn fp_reg() -> Rc<RefCell<RISCVValue>> {
        Rc::new(RefCell::new(RISCVValue::IReg(8)))
    }

    pub fn new(xlen: usize) -> RISCVBuilder {
        RISCVBuilder {
            file_ids: vec![],
            spans: vec![],
            symtabs: vec![],
            globals: IndexMap::new(),
            text_section: IndexMap::new(),
            cur_block_name: String::new(),
            func_values: vec![],
            values: HashMap::new(),
            ireg_num: 32,
            freg_num: 32,
            xlen,
        }
    }

    pub fn add_instruction(
        &mut self,
        opcode: impl ToString,
        operands: &[Rc<RefCell<RISCVValue>>],
    ) -> Result<(), Diagnostic<usize>> {
        let cur_basic_block = self.current_basic_block()?;

        let BasicBlock {
            name: _,
            instructions,
            cursor,
        } = &mut *cur_basic_block.borrow_mut();

        instructions.insert(
            *cursor,
            Instruction {
                opcode: opcode.to_string(),
                operands: operands.to_vec(),
            },
        );
        *cursor += 1;

        Ok(())
    }

    pub fn assign_ireg(&mut self) -> Rc<RefCell<RISCVValue>> {
        let a = Rc::new(RefCell::new(RISCVValue::IReg(self.ireg_num)));
        self.ireg_num += 1;
        a
    }

    pub fn assign_freg(&mut self) -> Rc<RefCell<RISCVValue>> {
        let a = Rc::new(RefCell::new(RISCVValue::FReg(self.freg_num)));
        self.freg_num += 1;
        a
    }

    pub fn extract_real_imag(
        &mut self,
        value: &Rc<RefCell<RISCVValue>>,
        r#type: &Rc<RefCell<Type>>,
    ) -> Result<(Rc<RefCell<RISCVValue>>, Rc<RefCell<RISCVValue>>), Diagnostic<usize>> {
        match &r#type.borrow().kind {
            TypeKind::Float => Ok((value.clone(), self.load_float(&RISCVValue::Float(0.))?)),
            TypeKind::Double => Ok((value.clone(), self.load_float(&RISCVValue::Double(0.))?)),
            TypeKind::Complex(Some(_)) => match &*value.borrow() {
                RISCVValue::Struct(t) => Ok((t[0].clone(), t[1].clone())),
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }

    pub fn to_bool(
        &mut self,
        value: &Rc<RefCell<RISCVValue>>,
        r#type: &Rc<RefCell<Type>>,
    ) -> Result<Rc<RefCell<RISCVValue>>, Diagnostic<usize>> {
        match &r#type.borrow().kind {
            t if t.is_integer() && t.is_unsigned().unwrap() || t.is_pointer() => self.binop(
                BinOpKind::Eq,
                value,
                r#type,
                &Rc::new(RefCell::new(RISCVValue::Integer(0))),
                r#type,
            ),
            t if t.is_integer() => self.binop(
                BinOpKind::Eq,
                value,
                r#type,
                &Rc::new(RefCell::new(RISCVValue::Integer(0))),
                r#type,
            ),
            t if t.is_float() => {
                let t = self.load_float(&RISCVValue::Float(0.))?;
                self.binop(BinOpKind::Eq, value, r#type, &t, r#type)
            }
            t if t.is_double() => {
                let t = self.load_float(&RISCVValue::Double(0.))?;
                self.binop(BinOpKind::Eq, value, r#type, &t, r#type)
            }
            t if t.is_complex() => {
                let TypeKind::Complex(Some(part_type)) = t else {
                    unreachable!()
                };
                match &*value.borrow() {
                    RISCVValue::Struct(t) => {
                        let t1 = self.to_bool(&t[0], part_type)?;
                        let t2 = self.to_bool(&t[1], part_type)?;
                        let rd = self.assign_ireg();
                        self.add_instruction("and", &[rd.clone(), t1, t2])?;
                        Ok(rd)
                    }
                    _ => unreachable!(),
                }
            }
            //TODO 十进制浮点数
            _ => todo!(),
        }
    }

    pub fn load_float(
        &mut self,
        value: &RISCVValue,
    ) -> Result<Rc<RefCell<RISCVValue>>, Diagnostic<usize>> {
        match value {
            RISCVValue::Float(value) => {
                let rd = self.assign_freg();
                let t1 = self.assign_ireg();
                self.add_instruction(
                    "li",
                    &[
                        t1.clone(),
                        Rc::new(RefCell::new(RISCVValue::Integer(value.to_bits() as u64))),
                    ],
                )?;
                self.add_instruction("fmv.s.x", &[rd.clone(), t1.clone()])?;
                Ok(rd)
            }
            RISCVValue::Double(value) => {
                let rd = self.assign_freg();
                let t1 = self.assign_ireg();
                self.add_instruction(
                    "li",
                    &[
                        t1.clone(),
                        Rc::new(RefCell::new(RISCVValue::Integer(value.to_bits() as u64))),
                    ],
                )?;
                self.add_instruction("fmv.d.x", &[rd.clone(), t1.clone()])?;
                Ok(rd)
            }
            //TODO long double
            _ => todo!(),
        }
    }

    pub fn build_int_add(
        &mut self,
        a: &Rc<RefCell<RISCVValue>>,
        b: &Rc<RefCell<RISCVValue>>,
    ) -> Result<Rc<RefCell<RISCVValue>>, Diagnostic<usize>> {
        match (&*a.borrow(), &*b.borrow()) {
            //超出单个寄存器长度的整数, 比如_BitInt
            (RISCVValue::Array(a), RISCVValue::Array(b)) => {
                let mut rd = vec![];
                //进行整数提升后a和b长度一样
                for _ in 0..a.len() {
                    let t = self.assign_ireg();
                    self.add_instruction(
                        "li",
                        &[t.clone(), Rc::new(RefCell::new(RISCVValue::Integer(0)))],
                    )?;
                    rd.push(t);
                }
                for (i, (a, b)) in a.iter().zip(b).enumerate() {
                    if i == 0 {
                        self.add_instruction("add", &[rd[0].clone(), a.clone(), b.clone()])?;
                    } else {
                        //可能有进位, 所以要逐个相加
                        self.add_instruction("add", &[rd[i].clone(), rd[i].clone(), a.clone()])?;
                        self.add_instruction("add", &[rd[i].clone(), rd[i].clone(), b.clone()])?;
                    }
                    if i < rd.len() - 1 {
                        self.add_instruction(
                            "sltu",
                            &[rd[i + 1].clone(), rd[i].clone(), a.clone()],
                        )?;
                    }
                }
                Ok(Rc::new(RefCell::new(RISCVValue::Array(rd))))
            }
            //这种情况出现在代码生成时, 为了方便而调用
            //实际的C代码会因为整数提升让a, b长度一样
            (RISCVValue::Array(a), RISCVValue::Integer(_)) => {
                let mut rd = vec![];
                for _ in 0..a.len() {
                    let t = self.assign_ireg();
                    self.add_instruction(
                        "li",
                        &[t.clone(), Rc::new(RefCell::new(RISCVValue::Integer(0)))],
                    )?;
                    rd.push(t);
                }
                for (i, a) in a.iter().enumerate() {
                    if i == 0 {
                        self.add_instruction("add", &[rd[0].clone(), a.clone(), b.clone()])?;
                    } else {
                        //可能有进位, 所以要逐个相加
                        self.add_instruction("add", &[rd[i].clone(), rd[i].clone(), a.clone()])?;
                    }
                    if i < rd.len() - 1 {
                        self.add_instruction(
                            "sltu",
                            &[rd[i + 1].clone(), rd[i].clone(), a.clone()],
                        )?;
                    }
                }
                Ok(Rc::new(RefCell::new(RISCVValue::Array(rd))))
            }
            _ => {
                let rd = self.assign_ireg();
                self.add_instruction("add", &[rd.clone(), a.clone(), b.clone()])?;
                Ok(rd)
            }
        }
    }

    pub fn build_int_sub(
        &mut self,
        a: &Rc<RefCell<RISCVValue>>,
        b: &Rc<RefCell<RISCVValue>>,
    ) -> Result<Rc<RefCell<RISCVValue>>, Diagnostic<usize>> {
        match (&*a.borrow(), &*b.borrow()) {
            //超出单个寄存器长度的整数, 比如_BitInt
            (RISCVValue::Array(a), RISCVValue::Array(b)) => {
                let mut rd = vec![];
                //进行整数提升后a和b长度一样
                for _ in 0..a.len() {
                    let t = self.assign_ireg();
                    self.add_instruction(
                        "li",
                        &[t.clone(), Rc::new(RefCell::new(RISCVValue::Integer(0)))],
                    )?;
                    rd.push(t);
                }
                for (i, (a, b)) in a.iter().zip(b).enumerate() {
                    if i == 0 {
                        self.add_instruction("sub", &[rd[0].clone(), a.clone(), b.clone()])?;
                    } else {
                        //可能有借位, 所以要逐个相减
                        self.add_instruction("sub", &[rd[i].clone(), rd[i].clone(), a.clone()])?;
                        self.add_instruction("sub", &[rd[i].clone(), rd[i].clone(), b.clone()])?;
                    }
                    if i < rd.len() - 1 {
                        self.add_instruction("sltu", &[rd[i + 1].clone(), a.clone(), b.clone()])?;
                        //转成-1或0
                        self.add_instruction("neg", &[rd[i + 1].clone()])?;
                    }
                }
                Ok(Rc::new(RefCell::new(RISCVValue::Array(rd))))
            }
            _ => {
                let rd = self.assign_ireg();
                self.add_instruction("sub", &[rd.clone(), a.clone(), b.clone()])?;
                Ok(rd)
            }
        }
    }

    pub fn build_int_compare(
        &mut self,
        op: BinOpKind,
        a: &Rc<RefCell<RISCVValue>>,
        b: &Rc<RefCell<RISCVValue>>,
        is_unsigned: bool,
    ) -> Result<Rc<RefCell<RISCVValue>>, Diagnostic<usize>> {
        let opcode = match op {
            BinOpKind::Ge => "sgeu",
            BinOpKind::Gt => "sgtu",
            BinOpKind::Le => "sleu",
            BinOpKind::Lt => "sltu",
            _ => unreachable!(),
        };
        match (&*a.borrow(), &*b.borrow()) {
            //超出单个寄存器长度的整数, 比如_BitInt
            (RISCVValue::Array(a), RISCVValue::Array(b)) => {
                let rd = self.assign_ireg();
                let t = self.assign_ireg();
                self.add_instruction(
                    "li",
                    &[rd.clone(), Rc::new(RefCell::new(RISCVValue::Integer(0)))],
                )?;
                for (i, (a, b)) in a.iter().zip(b).enumerate().rev() {
                    self.add_instruction(
                        //最高位需要考虑符号
                        if i == 0 && !is_unsigned {
                            &opcode[..3]
                        } else {
                            opcode
                        },
                        &[t.clone(), a.clone(), b.clone()],
                    )?;
                    self.add_instruction("and", &[rd.clone(), rd.clone(), t.clone()])?;
                }
                Ok(rd)
            }
            _ => {
                let rd = self.assign_ireg();
                self.add_instruction(opcode, &[rd.clone(), a.clone(), b.clone()])?;
                Ok(rd)
            }
        }
    }

    pub fn abs(
        &mut self,
        a: &Rc<RefCell<RISCVValue>>,
    ) -> Result<Rc<RefCell<RISCVValue>>, Diagnostic<usize>> {
        match &*a.borrow() {
            RISCVValue::Array(x) => {
                let mut rd = vec![];
                for _ in 0..x.len() {
                    rd.push(self.assign_ireg());
                }

                let t = self.assign_ireg();
                self.add_instruction(
                    "sra",
                    &[
                        t.clone(),
                        x[x.len() - 1].clone(),
                        Rc::new(RefCell::new(RISCVValue::Integer((self.xlen - 1) as u64))),
                    ],
                )?;

                //取反
                for (i, v) in x.iter().enumerate() {
                    self.add_instruction("xor", &[rd[i].clone(), v.clone(), t.clone()])?;
                }

                //获得符号
                self.add_instruction(
                    "and",
                    &[
                        t.clone(),
                        t.clone(),
                        Rc::new(RefCell::new(RISCVValue::Integer(1))),
                    ],
                )?;

                Ok(self.build_int_add(&Rc::new(RefCell::new(RISCVValue::Array(rd))), &t)?)
            }
            _ => {
                let rd = self.assign_ireg();
                let t = self.assign_ireg();
                self.add_instruction(
                    "sra",
                    &[
                        t.clone(),
                        a.clone(),
                        Rc::new(RefCell::new(RISCVValue::Integer((self.xlen - 1) as u64))),
                    ],
                )?;
                //取反
                self.add_instruction("xor", &[rd.clone(), a.clone(), t.clone()])?;
                //获得符号
                self.add_instruction(
                    "and",
                    &[
                        t.clone(),
                        t.clone(),
                        Rc::new(RefCell::new(RISCVValue::Integer(1))),
                    ],
                )?;
                self.add_instruction("add", &[rd.clone(), rd.clone(), t.clone()])?;
                Ok(rd)
            }
        }
    }

    pub fn build_int_mul(
        &mut self,
        a: &Rc<RefCell<RISCVValue>>,
        b: &Rc<RefCell<RISCVValue>>,
        is_unsigned: bool,
    ) -> Result<Rc<RefCell<RISCVValue>>, Diagnostic<usize>> {
        match (&*a.borrow(), &*b.borrow()) {
            //超出单个寄存器长度的整数, 比如_BitInt
            (RISCVValue::Array(_), RISCVValue::Array(_)) => {
                let (a, b) = if is_unsigned {
                    (a, b)
                } else {
                    (&self.abs(a)?, &self.abs(b)?)
                };

                let (RISCVValue::Array(a), RISCVValue::Array(b)) = (&*a.borrow(), &*b.borrow())
                else {
                    unreachable!()
                };

                let mut rd = vec![];
                let t = self.assign_ireg();

                for _ in 0..(a.len() + b.len()) {
                    let t = self.assign_ireg();
                    self.add_instruction(
                        "li",
                        &[t.clone(), Rc::new(RefCell::new(RISCVValue::Integer(0)))],
                    )?;
                    rd.push(t);
                }

                let mut i = 0;
                while i < a.len() {
                    let x = a[i].clone();
                    let mut j = 0;
                    while j < b.len() {
                        let y = b[j].clone();
                        let k = i + j;

                        self.add_instruction("mul", &[t.clone(), x.clone(), y.clone()])?;
                        self.add_instruction("add", &[rd[k].clone(), rd[k].clone(), t.clone()])?;
                        self.add_instruction("mulhu", &[t.clone(), x.clone(), y.clone()])?;
                        self.add_instruction(
                            "add",
                            &[rd[k + 1].clone(), rd[k + 1].clone(), t.clone()],
                        )?;

                        j += 1;
                    }
                    i += 1;
                }

                if !is_unsigned {
                    let a_sign = self.assign_ireg();
                    let b_sign = self.assign_ireg();

                    let word_size_m1 =
                        Rc::new(RefCell::new(RISCVValue::Integer((self.xlen - 1) as u64)));

                    self.add_instruction(
                        "srl",
                        &[a_sign.clone(), a[a.len() - 1].clone(), word_size_m1.clone()],
                    )?;
                    self.add_instruction(
                        "srl",
                        &[b_sign.clone(), b[b.len() - 1].clone(), word_size_m1.clone()],
                    )?;

                    self.add_instruction("xor", &[t.clone(), a_sign.clone(), b_sign.clone()])?;
                    self.add_instruction("sll", &[t.clone(), t.clone(), word_size_m1.clone()])?;
                    self.add_instruction(
                        "or",
                        &[
                            rd[rd.len() - 1].clone(),
                            rd[rd.len() - 1].clone(),
                            t.clone(),
                        ],
                    )?;
                }

                Ok(Rc::new(RefCell::new(RISCVValue::Array(rd))))
            }
            _ => {
                let rd = self.assign_ireg();
                self.add_instruction("mul", &[rd.clone(), a.clone(), b.clone()])?;
                Ok(rd)
            }
        }
    }

    pub fn build_int_bitand(
        &mut self,
        a: &Rc<RefCell<RISCVValue>>,
        b: &Rc<RefCell<RISCVValue>>,
    ) -> Result<Rc<RefCell<RISCVValue>>, Diagnostic<usize>> {
        match (&*a.borrow(), &*b.borrow()) {
            //超出单个寄存器长度的整数, 比如_BitInt
            (RISCVValue::Array(x), RISCVValue::Array(y)) => {
                let mut rd = vec![];
                //进行整数提升后a和b长度一样
                for _ in 0..x.len() {
                    rd.push(self.assign_ireg());
                }
                for (i, (x, y)) in x.iter().zip(y).enumerate() {
                    self.add_instruction("and", &[rd[i].clone(), x.clone(), y.clone()])?;
                }
                Ok(Rc::new(RefCell::new(RISCVValue::Array(rd))))
            }
            _ => {
                let rd = self.assign_ireg();
                self.add_instruction("and", &[rd.clone(), a.clone(), b.clone()])?;
                Ok(rd)
            }
        }
    }

    pub fn build_int_bitor(
        &mut self,
        a: &Rc<RefCell<RISCVValue>>,
        b: &Rc<RefCell<RISCVValue>>,
    ) -> Result<Rc<RefCell<RISCVValue>>, Diagnostic<usize>> {
        match (&*a.borrow(), &*b.borrow()) {
            //超出单个寄存器长度的整数, 比如_BitInt
            (RISCVValue::Array(x), RISCVValue::Array(y)) => {
                let mut rd = vec![];
                //进行整数提升后a和b长度一样
                for _ in 0..x.len() {
                    rd.push(self.assign_ireg());
                }
                for (i, (x, y)) in x.iter().zip(y).enumerate() {
                    self.add_instruction("or", &[rd[i].clone(), x.clone(), y.clone()])?;
                }
                Ok(Rc::new(RefCell::new(RISCVValue::Array(rd))))
            }
            _ => {
                let rd = self.assign_ireg();
                self.add_instruction("or", &[rd.clone(), a.clone(), b.clone()])?;
                Ok(rd)
            }
        }
    }

    pub fn build_int_left_shift(
        &mut self,
        a: &Rc<RefCell<RISCVValue>>,
        b: &Rc<RefCell<RISCVValue>>,
    ) -> Result<Rc<RefCell<RISCVValue>>, Diagnostic<usize>> {
        match (&*a.borrow(), &*b.borrow()) {
            //超出单个寄存器长度的整数, 比如_BitInt
            (RISCVValue::Array(x), RISCVValue::Array(y)) => {
                let mut rd = vec![];
                //进行整数提升后a和b长度一样
                for _ in 0..x.len() {
                    rd.push(self.assign_ireg());
                }
                for (i, (x, y)) in x.iter().zip(y).enumerate() {
                    self.add_instruction("sll", &[rd[i].clone(), x.clone(), y.clone()])?;
                }
                Ok(Rc::new(RefCell::new(RISCVValue::Array(rd))))
            }
            _ => {
                let rd = self.assign_ireg();
                self.add_instruction("sll", &[rd.clone(), a.clone(), b.clone()])?;
                Ok(rd)
            }
        }
    }

    //获得函数每个参数和返回值对应的存储位置
    pub fn call_convention(
        &self,
        r#type: &Rc<RefCell<Type>>,
    ) -> Result<(Vec<Rc<RefCell<RISCVValue>>>, Rc<RefCell<RISCVValue>>), Diagnostic<usize>> {
        todo!()
    }
}
