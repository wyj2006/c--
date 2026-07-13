pub mod basic_block;
pub mod display;
pub mod function;
pub mod gen_decl;
pub mod gen_expr;
pub mod gen_init;
pub mod gen_stmt;
pub mod instruction;
#[cfg(test)]
pub mod tests;

use codespan_reporting::diagnostic::Diagnostic;
use indexmap::IndexMap;
use num::ToPrimitive;

use crate::{
    ast::TranslationUnit,
    codegen::riscv::{
        basic_block::BasicBlock,
        function::Function,
        instruction::{Instruction, Opcode, Operand},
    },
    ctype::{
        Type,
        layout::{ConstDesignation, compute_layout},
    },
    symtab::{Namespace, Symbol, SymbolKind, SymbolTable},
    variant::{Variant, to_decimal},
};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub struct CodeGen {
    pub symtabs: Vec<Rc<RefCell<SymbolTable>>>,
    //键是函数对应符号表项的指针值
    pub functions: IndexMap<usize, Function>,
    //当前函数在self.functions中的索引
    pub cur_function: usize,
    pub symbol_values: HashMap<usize, Operand>,
    //语句创建的标签, 索引0用于break, 索引1用于continue
    pub stmt_labels: HashMap<usize, Vec<Operand>>,
    pub ireg_num: usize,
    pub freg_num: usize,
    pub xlen: usize,
    pub globals: IndexMap<String, (Option<Variant>, Rc<RefCell<Type>>)>,
}

const RA_REG_INDEX: usize = 1;
const SP_REG_INDEX: usize = 2;
const FP_REG_INDEX: usize = 8;

const SP_REG: Operand = Operand::IntReg(2);
const FP_REG: Operand = Operand::IntReg(8);
const A0_REG: Operand = Operand::IntReg(10);
const A1_REG: Operand = Operand::IntReg(11);
const A2_REG: Operand = Operand::IntReg(12);
const FA0_REG: Operand = Operand::FPReg(10);

impl CodeGen {
    pub fn new() -> CodeGen {
        CodeGen {
            symtabs: vec![],
            functions: IndexMap::new(),
            cur_function: usize::MAX,
            symbol_values: HashMap::new(),
            stmt_labels: HashMap::new(),
            ireg_num: 32,
            freg_num: 32,
            xlen: 64,
            globals: IndexMap::new(),
        }
    }

    pub fn r#gen(&mut self, ast: &Rc<RefCell<TranslationUnit>>) -> Result<(), Diagnostic<usize>> {
        if let Some(symtab) = &ast.borrow().symtab {
            self.enter_scope(Rc::clone(symtab));
        }
        for decl in &ast.borrow().decls {
            self.visit_declaration(decl)?;
        }
        if let Some(_) = ast.borrow().symtab {
            self.leave_scope();
        }
        Ok(())
    }

    pub fn enter_scope(&mut self, symtab: Rc<RefCell<SymbolTable>>) {
        self.symtabs.push(symtab);
    }

    pub fn leave_scope(&mut self) {
        self.symtabs.pop();
    }

    pub fn lookup(&self, namespace: Namespace, name: &str) -> Option<Rc<RefCell<Symbol>>> {
        self.symtabs
            .last()
            .unwrap()
            .borrow()
            .lookup(namespace, name)
    }

    ///在当前函数的当前位置后插入一个basic block, 并返回这个block的名称
    pub fn append_basic_block(&mut self, name: &str) -> Result<String, Diagnostic<usize>> {
        let (_, function) = self.functions.get_index_mut(self.cur_function).unwrap();

        let base_name = name;
        let mut name = base_name.to_string();
        let mut i = 2;
        while function.basic_blocks.contains_key(&name) {
            name = format!("{base_name}{i}");
            i += 1;
        }

        function.basic_blocks.insert_before(
            function.cursor + 1,
            name.to_string(),
            BasicBlock::new(&name),
        );

        Ok(name)
    }

    pub fn position_at_end(&mut self, name: &str) -> Result<(), Diagnostic<usize>> {
        let (_, function) = self.functions.get_index_mut(self.cur_function).unwrap();
        let (index, _, basic_block) = function.basic_blocks.get_full_mut(name).unwrap();

        function.cursor = index;
        basic_block.cursor = basic_block.instructions.len();

        Ok(())
    }

    //将操作数转换成指令实际能接受的操作数
    pub fn normalize(&mut self, operand: &Operand) -> Result<Operand, Diagnostic<usize>> {
        match operand {
            Operand::Address { base, offset } => {
                let value = self.assign_ireg()?;
                self.add_instruction(
                    Opcode::Add,
                    &[value.clone(), (**base).clone(), Operand::Immediate(*offset)],
                )?;
                Ok(value)
            }
            Operand::Symbol(name) => {
                let value = self.assign_ireg()?;
                self.add_instruction(
                    Opcode::LoadAddr,
                    &[value.clone(), Operand::Symbol(name.clone())],
                )?;
                Ok(value)
            }
            Operand::Immediate(_) => {
                let t = self.assign_ireg()?;
                self.add_instruction(Opcode::LoadImm, &[t.clone(), operand.clone()])?;
                Ok(t)
            }
            _ => Ok(operand.clone()),
        }
    }

    pub fn add_instruction(
        &mut self,
        opcode: Opcode,
        operands: &[Operand],
    ) -> Result<(), Diagnostic<usize>> {
        let mut operands = operands.to_vec();

        match opcode {
            Opcode::LoadB
            | Opcode::LoadBU
            | Opcode::LoadD
            | Opcode::LoadH
            | Opcode::LoadHU
            | Opcode::LoadW
            | Opcode::LoadWU
            | Opcode::StoreB
            | Opcode::StoreD
            | Opcode::StoreH
            | Opcode::StoreW
            | Opcode::FStoreD
            | Opcode::FStoreS => operands[0] = self.normalize(&operands[0])?,
            Opcode::BEq | Opcode::BEqZ | Opcode::BNeqZ => {
                for operand in &mut operands[..2] {
                    *operand = self.normalize(operand)?;
                }
            }
            Opcode::Call | Opcode::Jump | Opcode::LoadAddr | Opcode::LoadImm => {}
            //这些指令都允许常量运算符
            Opcode::Add
            | Opcode::SetLt
            | Opcode::SetLtU
            | Opcode::Xor
            | Opcode::Or
            | Opcode::And
            | Opcode::LShift
            | Opcode::RShiftL
            | Opcode::RShiftA => {
                for operand in &mut operands[..2] {
                    *operand = self.normalize(operand)?;
                }
            }
            _ => {
                for operand in &mut operands {
                    *operand = self.normalize(operand)?;
                }
            }
        }

        let (_, function) = self.functions.get_index_mut(self.cur_function).unwrap();
        let (_, basic_block) = function
            .basic_blocks
            .get_index_mut(function.cursor)
            .unwrap();

        basic_block
            .instructions
            .insert(basic_block.cursor, Instruction { opcode, operands });
        basic_block.cursor += 1;

        Ok(())
    }

    pub fn current_basic_block(&self) -> String {
        let (_, function) = self.functions.get_index(self.cur_function).unwrap();
        function
            .basic_blocks
            .get_index(function.cursor)
            .unwrap()
            .0
            .clone()
    }

    pub fn variant_to_operand(
        &mut self,
        variant: &Variant,
        r#type: &Rc<RefCell<Type>>,
    ) -> Result<Operand, Diagnostic<usize>> {
        //TODO variant的类型可能不等于type
        match variant {
            Variant::Bool(a) => Ok(Operand::Immediate(*a as i64)),
            Variant::Nullptr => Ok(Operand::Immediate(0)),
            Variant::Int(a) => Ok(Operand::Immediate(a.to_i64().unwrap_or(i64::MAX))),
            Variant::Rational(a) => {
                let value_str = to_decimal(a).to_string();
                let t = self.assign_ireg()?;
                self.add_instruction(
                    Opcode::LoadImm,
                    &[
                        t.clone(),
                        Operand::Immediate(
                            value_str.parse::<f64>().unwrap_or(f64::MAX).to_bits() as i64
                        ),
                    ],
                )?;

                let value = self.assign_freg()?;
                self.add_instruction(
                    if r#type.borrow().is_float_type() {
                        match self.xlen {
                            32 => Opcode::FMoveSW,
                            64 => Opcode::FMoveSL,
                            _ => unreachable!(),
                        }
                    } else {
                        match self.xlen {
                            32 => Opcode::FMoveDW,
                            64 => Opcode::FMoveDL,
                            _ => unreachable!(),
                        }
                    },
                    &[value.clone(), t],
                )?;
                Ok(value)
            }
            _ => Err(Diagnostic::error()),
        }
    }

    pub fn assign_ireg(&mut self) -> Result<Operand, Diagnostic<usize>> {
        self.ireg_num += 1;
        Ok(Operand::IntReg((self.ireg_num - 1) as u64))
    }

    pub fn assign_freg(&mut self) -> Result<Operand, Diagnostic<usize>> {
        self.freg_num += 1;
        Ok(Operand::FPReg((self.freg_num - 1) as u64))
    }

    pub fn load(
        &mut self,
        ptr: &Operand,
        r#type: &Rc<RefCell<Type>>,
        symbol: &Option<Rc<RefCell<Symbol>>>,
    ) -> Result<Operand, Diagnostic<usize>> {
        match &r#type.borrow().kind {
            t if t.is_float_type() => {
                let value = self.assign_freg()?;
                self.add_instruction(Opcode::FLoadS, &[value.clone(), ptr.clone()])?;
                Ok(value)
            }
            t if t.is_double() => {
                let value = self.assign_freg()?;
                self.add_instruction(Opcode::FLoadD, &[value.clone(), ptr.clone()])?;
                Ok(value)
            }
            t if t.is_scale() => {
                let size = t.size().unwrap();
                let unsigned = t.is_unsigned().unwrap_or(true);
                let value = self.assign_ireg()?;

                let symbol = match symbol {
                    Some(symbol) => Some((*symbol.borrow()).clone()),
                    None => None,
                };

                if let Some(symbol) = symbol
                    && let Symbol {
                        name,
                        kind:
                            SymbolKind::Member {
                                index,
                                belong_record,
                                ..
                            },
                        ..
                    } = symbol
                {
                    let layout = compute_layout(Rc::clone(&belong_record)).unwrap();
                    let layout = &layout.children[index];
                    for child in &layout.children {
                        if let Some(ConstDesignation::MemberAccess(t)) = &child.designation
                            && *t == *name
                        {
                            self.add_instruction(
                                match (
                                    layout.r#type.borrow().size().unwrap(),
                                    layout.r#type.borrow().is_unsigned().unwrap(),
                                ) {
                                    (1, true) => Opcode::LoadBU,
                                    (1, false) => Opcode::LoadB,
                                    (2, true) => Opcode::LoadHU,
                                    (2, false) => Opcode::LoadH,
                                    (4, true) => Opcode::LoadWU,
                                    (4, false) => Opcode::LoadW,
                                    (8, _) => Opcode::LoadD,
                                    _ => unreachable!(),
                                },
                                &[value.clone(), ptr.clone()],
                            )?;

                            self.add_instruction(
                                Opcode::RShiftL,
                                &[
                                    value.clone(),
                                    value.clone(),
                                    Operand::Immediate(child.offset as i64),
                                ],
                            )?;
                            self.add_instruction(
                                Opcode::And,
                                &[
                                    value.clone(),
                                    value.clone(),
                                    Operand::Immediate((1 << child.width) - 1),
                                ],
                            )?;

                            self.add_instruction(
                                Opcode::LShift,
                                &[
                                    value.clone(),
                                    value.clone(),
                                    Operand::Immediate((self.xlen - size) as i64),
                                ],
                            )?;
                            self.add_instruction(
                                if unsigned {
                                    Opcode::RShiftL
                                } else {
                                    Opcode::RShiftA
                                },
                                &[
                                    value.clone(),
                                    value.clone(),
                                    Operand::Immediate((self.xlen - size) as i64),
                                ],
                            )?;
                            break;
                        }
                    }
                } else {
                    self.add_instruction(
                        match (size, unsigned) {
                            (1, true) => Opcode::LoadBU,
                            (1, false) => Opcode::LoadB,
                            (2, true) => Opcode::LoadHU,
                            (2, false) => Opcode::LoadH,
                            (4, true) => Opcode::LoadWU,
                            (4, false) => Opcode::LoadW,
                            (8, _) => Opcode::LoadD,
                            _ => unreachable!(),
                        },
                        &[value.clone(), ptr.clone()],
                    )?;
                }
                Ok(value)
            }
            //对于复合类型, 将表示它们的值视作指针
            t if t.is_aggregate() => Ok(ptr.clone()),
            _ => unreachable!(),
        }
    }

    pub fn store(
        &mut self,
        ptr: &Operand,
        value: &Operand,
        r#type: &Rc<RefCell<Type>>,
        symbol: &Option<Rc<RefCell<Symbol>>>,
    ) -> Result<(), Diagnostic<usize>> {
        match &r#type.borrow().kind {
            t if t.is_float_type() => {
                self.add_instruction(Opcode::FStoreS, &[value.clone(), ptr.clone()])?;
            }
            t if t.is_double() => {
                self.add_instruction(Opcode::FStoreD, &[value.clone(), ptr.clone()])?;
            }
            t if t.is_scale() => {
                let mut size = t.size().unwrap();

                //防止后面在计算layout的时候reborrow
                let symbol = match symbol {
                    Some(symbol) => Some((*symbol.borrow()).clone()),
                    None => None,
                };

                if let Some(symbol) = symbol
                    && let Symbol {
                        name,
                        kind:
                            SymbolKind::Member {
                                index,
                                belong_record,
                                ..
                            },
                        ..
                    } = symbol
                {
                    let layout = compute_layout(Rc::clone(&belong_record)).unwrap();
                    let layout = &layout.children[index];
                    size = layout.r#type.borrow().size().unwrap();

                    for child in &layout.children {
                        if let Some(ConstDesignation::MemberAccess(t)) = &child.designation
                            && *t == *name
                        {
                            let old_value = self.load(ptr, &layout.r#type, &None)?;
                            self.add_instruction(
                                Opcode::And,
                                &[
                                    old_value.clone(),
                                    old_value.clone(),
                                    Operand::Immediate(!(((1 << child.width) - 1) << child.offset)),
                                ],
                            )?;
                            self.add_instruction(
                                Opcode::And,
                                &[
                                    value.clone(),
                                    value.clone(),
                                    Operand::Immediate((1 << child.width) - 1),
                                ],
                            )?;
                            self.add_instruction(
                                Opcode::LShift,
                                &[
                                    value.clone(),
                                    value.clone(),
                                    Operand::Immediate(child.offset as i64),
                                ],
                            )?;
                            self.add_instruction(
                                Opcode::Or,
                                &[value.clone(), value.clone(), old_value.clone()],
                            )?;

                            break;
                        }
                    }
                }
                self.add_instruction(
                    match size {
                        1 => Opcode::StoreB,
                        2 => Opcode::StoreH,
                        4 => Opcode::StoreW,
                        8 => Opcode::StoreD,
                        _ => unreachable!(),
                    },
                    &[value.clone(), ptr.clone()],
                )?;
            }
            //对于复合类型, 将表示它们的值视作指针
            t if t.is_aggregate() => {
                let size = t.size().unwrap();
                self.call_memcpy(value, ptr, &Operand::Immediate(size as i64))?;
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    pub fn to_bool(
        &mut self,
        value: &Operand,
        //主要用于判断是float还是double
        r#type: Option<&Rc<RefCell<Type>>>,
    ) -> Result<Operand, Diagnostic<usize>> {
        match value {
            t @ (Operand::Immediate(_) | Operand::IntReg(_)) => {
                let value = self.assign_ireg()?;
                self.add_instruction(Opcode::SetNeqZ, &[value.clone(), t.clone()])?;
                Ok(value)
            }
            Operand::Address { base, offset } => {
                let value = self.assign_ireg()?;
                self.add_instruction(
                    Opcode::Add,
                    &[value.clone(), (**base).clone(), Operand::Immediate(*offset)],
                )?;
                self.to_bool(&value, None)
            }
            t @ Operand::Symbol(_) => {
                let value = self.assign_ireg()?;
                self.add_instruction(Opcode::LoadAddr, &[value.clone(), t.clone()])?;
                self.to_bool(&value, None)
            }
            t @ Operand::FPReg(_) => {
                let Some(r#type) = r#type else { unreachable!() };

                let value = self.assign_ireg()?;
                let zero = self.assign_freg()?;

                if r#type.borrow().is_float_type() {
                    self.add_instruction(
                        Opcode::LoadImm,
                        &[value.clone(), Operand::Immediate(0.0f32.to_bits() as i64)],
                    )?;
                    self.add_instruction(
                        match self.xlen {
                            32 => Opcode::FMoveSW,
                            64 => Opcode::FMoveSL,
                            _ => unreachable!(),
                        },
                        &[zero.clone(), value.clone()],
                    )?;
                    self.add_instruction(Opcode::FEqS, &[value.clone(), t.clone(), zero])?;
                } else {
                    self.add_instruction(
                        Opcode::LoadImm,
                        &[value.clone(), Operand::Immediate(0.0f64.to_bits() as i64)],
                    )?;
                    self.add_instruction(
                        match self.xlen {
                            32 => Opcode::FMoveDW,
                            64 => Opcode::FMoveDL,
                            _ => unreachable!(),
                        },
                        &[zero.clone(), value.clone()],
                    )?;
                    self.add_instruction(Opcode::FEqD, &[value.clone(), t.clone(), zero])?;
                }

                Ok(value)
            }
        }
    }

    pub fn call_memcpy(
        &mut self,
        dst: &Operand,
        src: &Operand,
        n: &Operand,
    ) -> Result<Operand, Diagnostic<usize>> {
        self.add_instruction(Opcode::Move, &[A0_REG, dst.clone()])?;
        self.add_instruction(Opcode::Move, &[A1_REG, src.clone()])?;
        self.add_instruction(Opcode::Move, &[A2_REG, n.clone()])?;
        self.add_instruction(Opcode::Call, &[Operand::Symbol("memcpy".to_string())])?;
        Ok(dst.clone())
    }

    pub fn call_memset(
        &mut self,
        src: &Operand,
        c: &Operand,
        n: &Operand,
    ) -> Result<Operand, Diagnostic<usize>> {
        self.add_instruction(Opcode::Move, &[A0_REG, src.clone()])?;
        self.add_instruction(Opcode::Move, &[A1_REG, c.clone()])?;
        self.add_instruction(Opcode::Move, &[A2_REG, n.clone()])?;
        self.add_instruction(Opcode::Call, &[Operand::Symbol("memset".to_string())])?;
        Ok(src.clone())
    }

    pub fn add_global(
        &mut self,
        base_name: &str,
        value: (Option<Variant>, Rc<RefCell<Type>>),
    ) -> Result<String, Diagnostic<usize>> {
        let mut i = 2;
        let mut name = base_name.to_string();
        while self.globals.contains_key(&name) {
            name = format!("{base_name}{i}");
            i += 1;
        }
        self.globals.insert(name.clone(), value);
        Ok(name)
    }

    //处理需要在栈上传递的参数
    pub fn push_arg(
        &mut self,
        value: &Operand,
        r#type: &Rc<RefCell<Type>>,
        arg_frame_size: &mut usize,
    ) -> Result<(), Diagnostic<usize>> {
        let address = Operand::Address {
            base: Box::new(SP_REG),
            offset: (*arg_frame_size) as i64,
        };
        *arg_frame_size += r#type.borrow().size().unwrap();

        match &r#type.borrow().kind {
            //对于复合类型来说, push的是它的指针, 也有可能是它的一部分数据, 所以不能直接用self.store
            //但不管怎么样, 入栈的数据的大小一定是xlen
            t if t.is_aggregate() => self.add_instruction(
                match self.xlen {
                    32 => Opcode::StoreW,
                    64 => Opcode::StoreD,
                    _ => unreachable!(),
                },
                &[value.clone(), address],
            )?,
            _ => self.store(&address, value, r#type, &None)?,
        }
        Ok(())
    }

    pub fn pop_arg(&mut self, r#type: &Rc<RefCell<Type>>) -> Result<Operand, Diagnostic<usize>> {
        let (_, function) = self.functions.get_index_mut(self.cur_function).unwrap();
        let ptr = &Operand::Address {
            base: Box::new(FP_REG),
            offset: function.param_frame_size as i64,
        };
        function.param_frame_size += r#type.borrow().size().unwrap();

        match &r#type.borrow().kind {
            //对于复合类型来说, pop的是对应地址的值, 所以不能用self.load
            t if t.is_aggregate() => {
                let t = self.assign_ireg()?;
                self.add_instruction(
                    match self.xlen {
                        32 => Opcode::LoadWU,
                        64 => Opcode::LoadD,
                        _ => unreachable!(),
                    },
                    &[t.clone(), ptr.clone()],
                )?;
                Ok(t)
            }
            _ => self.load(ptr, r#type, &None),
        }
    }

    pub fn save_callee_regs(&mut self) -> Result<(), Diagnostic<usize>> {
        let (_, function) = self.functions.get_index_mut(self.cur_function).unwrap();
        for (ireg_index, frame_offset) in function.ireg_saved.clone() {
            self.add_instruction(
                match self.xlen {
                    32 => Opcode::StoreW,
                    64 => Opcode::StoreD,
                    _ => unreachable!(),
                },
                &[
                    Operand::IntReg(ireg_index as u64),
                    Operand::Address {
                        //假设这个时候fp和sp寄存器还没有改变
                        base: Box::new(SP_REG),
                        offset: frame_offset,
                    },
                ],
            )?;
        }
        Ok(())
    }

    pub fn restore_callee_regs(&mut self) -> Result<(), Diagnostic<usize>> {
        let (_, function) = self.functions.get_index_mut(self.cur_function).unwrap();
        for (ireg_index, frame_offset) in function.ireg_saved.clone() {
            self.add_instruction(
                match self.xlen {
                    32 => Opcode::LoadWU,
                    64 => Opcode::LoadD,
                    _ => unreachable!(),
                },
                &[
                    Operand::IntReg(ireg_index as u64),
                    Operand::Address {
                        base: Box::new(FP_REG),
                        offset: frame_offset,
                    },
                ],
            )?;
        }
        Ok(())
    }
}
