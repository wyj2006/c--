use crate::{
    ast::{
        decl::{FunctionSpec, StorageClass, StorageClassKind},
        expr::{BinOpKind, CastMethod, UnaryOpKind},
    },
    codegen::builder::{
        Builder,
        riscv::{RISCVBuilder, basic_block::BasicBlock, value::RISCVValue},
    },
    ctype::{
        RecordKind, Type, TypeKind, array_element, complex_part_type, get_inner_type,
        layout::{ConstDesignation, Layout},
        pointee,
    },
    symtab::{Namespace, SymbolTable},
    variant::{Variant, to_decimal},
};
use codespan::Span;
use codespan_reporting::diagnostic::{Diagnostic, Label};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

//TODO 可能在获取type的种类之前都需要调用get_inner_type

impl Builder for RISCVBuilder {
    type Value = Rc<RefCell<RISCVValue>>;
    type BasicBlock = Rc<RefCell<BasicBlock>>;

    fn append_context(&mut self, file_id: usize, span: Span) {
        self.file_ids.push(file_id);
        self.spans.push(span);
    }

    fn pop_context(&mut self) {
        self.file_ids.pop();
        self.spans.pop();
    }

    fn enter_scope(&mut self, symtab: &Rc<RefCell<SymbolTable>>) {
        self.symtabs.push(symtab.clone());
    }

    fn leave_scope(&mut self) {
        self.symtabs.pop();
    }

    fn lookup(
        &self,
        name: &str,
        namespace: crate::symtab::Namespace,
    ) -> Option<Rc<RefCell<crate::symtab::Symbol>>> {
        self.symtabs
            .last()
            .unwrap()
            .borrow()
            .lookup(namespace, name)
    }

    fn enter_function(&mut self, function: &Self::Value) -> Result<(), Diagnostic<usize>> {
        self.func_values.push(function.clone());

        match &*function.borrow() {
            RISCVValue::Function { name, .. } => {
                self.append_basic_block(name)?;
                //entry块前面的部分将作为函数序言
                let entry_block = self.append_basic_block("entry")?;
                self.position_at_end(&entry_block);
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    fn leave_function(&mut self, function: &Self::Value) -> Result<(), Diagnostic<usize>> {
        let RISCVValue::Function { name, .. } = &*function.borrow() else {
            unreachable!()
        };
        self.func_values.pop();

        let Some((mut index, _, _)) = self.text_section.get_full_mut(name) else {
            unreachable!()
        };
        index += 1;
        while let Some((_, t)) = self.text_section.get_index(index) {
            if t.borrow().name.starts_with("unreach") {
                self.text_section.shift_remove_index(index);
                continue;
            }
            index += 1;
        }
        //TODO 补充返回值
        Ok(())
    }

    fn append_basic_block(&mut self, name: &str) -> Result<Self::BasicBlock, Diagnostic<usize>> {
        let new_block = Rc::new(RefCell::new(BasicBlock::new(name)));

        //TODO 不需要移动到不同函数的基本块中
        if let Some((index, _, _)) = self.text_section.get_full(&self.cur_block_name) {
            self.text_section
                .insert_before(index + 1, name.to_string(), Rc::clone(&new_block));
        }

        Ok(new_block)
    }

    fn current_basic_block(&self) -> Result<Self::BasicBlock, Diagnostic<usize>> {
        Ok(self.text_section.get(&self.cur_block_name).unwrap().clone())
    }

    fn position_at_end(&self, basic_block: &Self::BasicBlock) {
        let BasicBlock {
            name: _,
            instructions,
            cursor,
        } = &mut *basic_block.borrow_mut();
        *cursor = instructions.len();
    }

    fn get_previous_block(&self, basic_block: &Self::BasicBlock) -> Option<Self::BasicBlock> {
        if let Some((index, _, _)) = self.text_section.get_full(&basic_block.borrow().name) {
            if index >= 1 {
                Some(self.text_section.get_index(index - 1).unwrap().1.clone())
            } else {
                None
            }
        } else {
            None
        }
    }

    fn variant_to_value(
        &mut self,
        variant: &Variant,
        r#type: &Rc<RefCell<Type>>,
    ) -> Result<Self::Value, Diagnostic<usize>> {
        match variant {
            Variant::Int(a) => Ok(Rc::new(RefCell::new(RISCVValue::Integer(
                *a.to_u64_digits().1.get(0).unwrap_or(&0),
            )))),
            Variant::Bool(a) => Ok(Rc::new(RefCell::new(RISCVValue::Integer(*a as u64)))),
            Variant::Nullptr => Ok(Rc::new(RefCell::new(RISCVValue::Integer(0)))),
            Variant::Array(a) => {
                let element_type = array_element(r#type.clone()).unwrap();
                let mut t = vec![];
                for i in a {
                    t.push(self.variant_to_value(i, &element_type)?);
                }
                Ok(Rc::new(RefCell::new(RISCVValue::Array(t))))
            }
            Variant::Rational(a) => {
                let value = if r#type.borrow().is_float() {
                    let value = to_decimal(a).to_string().parse::<f32>().unwrap();
                    value.to_bits() as u64
                } else if r#type.borrow().is_double() {
                    let value = to_decimal(a).to_string().parse::<f64>().unwrap();
                    value.to_bits()
                } else if r#type.borrow().is_long_double() {
                    //TODO long double
                    todo!()
                } else {
                    unreachable!()
                };
                let value = Rc::new(RefCell::new(RISCVValue::Integer(value)));
                if self.func_values.len() == 0 {
                    Ok(value)
                } else {
                    let rd = self.assign_freg();
                    let t = self.assign_ireg();
                    self.add_instruction("li", &[t.clone(), value.clone()])?;

                    let mut opcode = "fcvt".to_string();
                    //TODO long double
                    if r#type.borrow().is_float() {
                        opcode += ".s";
                    } else if r#type.borrow().is_double() {
                        opcode += ".d";
                    }

                    if self.xlen <= 32 {
                        opcode += "wu";
                    } else {
                        opcode += "lu";
                    }

                    self.add_instruction(opcode, &[rd.clone(), t.clone()])?;
                    Ok(rd)
                }
            }
            Variant::Complex(real, imag) => {
                let part_type = complex_part_type(r#type).unwrap();
                let real = self.variant_to_value(&Variant::Rational(real.clone()), &part_type)?;
                let imag = self.variant_to_value(&Variant::Rational(imag.clone()), &part_type)?;
                Ok(RISCVValue::new_complex(real, imag))
            }
            Variant::Unknown => Err(Diagnostic::error()
                .with_message(format!("unkown constant value"))
                .with_label(Label::primary(
                    r#type.borrow().file_id,
                    r#type.borrow().span,
                ))),
        }
    }

    fn layout_to_value(
        &mut self,
        layout: Layout,
        path: Vec<ConstDesignation>,
        init_values: &HashMap<Vec<ConstDesignation>, Self::Value>,
    ) -> Result<Self::Value, Diagnostic<usize>> {
        if let Some(t) = init_values.get(&path) {
            Ok(t.clone())
        }
        //不是Record或者数组却有children, 说明是位域
        else if !(layout.r#type.borrow().is_array() || layout.r#type.borrow().is_record())
            && layout.children.len() > 0
        {
            let mut bitfields = vec![];
            for child in layout.children {
                let mut path = path.clone();
                if let Some(t) = &child.designation {
                    path.push(t.clone());
                }
                if let Some(t) = init_values.get(&path) {
                    bitfields.push((t.clone(), child.width, child.offset));
                }
            }

            if bitfields.iter().all(|x| x.0.borrow().is_const()) {
                let mut value = 0;
                for (bitfield, width, offset) in bitfields {
                    let RISCVValue::Integer(bitfield) = &*bitfield.borrow() else {
                        unreachable!()
                    };
                    value |= (*bitfield & ((1 << width) - 1)) << offset;
                }
                Ok(Rc::new(RefCell::new(RISCVValue::Integer(value))))
            } else {
                let mut value = self.assign_ireg();
                self.add_instruction(
                    "li",
                    &[value.clone(), Rc::new(RefCell::new(RISCVValue::Integer(0)))],
                )?;
                for (bitfield, width, offset) in bitfields {
                    let mask = Rc::new(RefCell::new(RISCVValue::Integer((1 << width) - 1)));
                    let and = self.build_int_bitand(&bitfield, &mask)?;
                    let shl = self.build_int_left_shift(
                        &and,
                        &Rc::new(RefCell::new(RISCVValue::Integer(offset as u64))),
                    )?;
                    value = self.build_int_bitor(&value, &shl)?;
                }
                Ok(value)
            }
        } else if layout.r#type.borrow().is_array() || layout.r#type.borrow().is_record() {
            let mut fields = vec![];
            'outer: for child in layout.children {
                let mut path = path.clone();
                if let Some(t) = &child.designation {
                    path.push(t.clone());
                }
                if layout.r#type.borrow().is_union() {
                    for designation in init_values.keys() {
                        if designation.starts_with(&path) {
                            fields.push(self.layout_to_value(child.clone(), path, init_values)?);
                            break 'outer;
                        }
                    }
                } else {
                    fields.push(self.layout_to_value(child.clone(), path, init_values)?);
                }
            }

            Ok(Rc::new(RefCell::new(
                if layout.r#type.borrow().is_array() {
                    RISCVValue::Array(fields)
                } else {
                    RISCVValue::Struct(fields)
                },
            )))
        } else {
            Ok(Rc::new(RefCell::new(RISCVValue::Integer(0))))
        }
    }

    fn decl_variable(
        &mut self,
        name: &str,
        r#type: &Rc<RefCell<Type>>,
        storage_classes: &[StorageClass],
        init_value: Option<Self::Value>,
        vla_size: Option<Self::Value>, //vla的长度
    ) -> Result<Self::Value, Diagnostic<usize>> {
        //TODO vla, thread_local
        let decl_global = self.func_values.len() == 0
            || storage_classes
                .iter()
                .any(|x| x.kind == StorageClassKind::Static);
        let value = if decl_global {
            self.globals.insert(name.to_string(), init_value.clone());
            Rc::new(RefCell::new(RISCVValue::Symbol(name.to_string())))
        } else {
            let size = r#type.borrow().size().unwrap();
            let RISCVValue::Function { frame_size, .. } =
                &mut *self.func_values.last_mut().unwrap().borrow_mut()
            else {
                unreachable!();
            };

            let address = Rc::new(RefCell::new(RISCVValue::Address(
                Self::fp_reg(),
                (*frame_size) as isize,
            )));
            *frame_size += size.next_power_of_two();
            address
        };

        if let Some(t) = init_value
            && !decl_global
        {
            self.store(&value, &t, r#type)?;
        }

        self.values.insert(
            self.lookup(name, Namespace::Ordinary).unwrap().as_ptr() as usize,
            value.clone(),
        );
        Ok(value)
    }

    fn decl_function(
        &mut self,
        name: &str,
        r#type: &Rc<RefCell<Type>>,
        storage_classes: &[StorageClass],
        function_specs: &[FunctionSpec],
    ) -> Result<Self::Value, Diagnostic<usize>> {
        let symbol = self.lookup(name, Namespace::Ordinary).unwrap();
        let key = symbol.as_ptr() as usize;

        //TODO 函数声明
        let function = if let Some(t) = self.values.get(&key) {
            t.clone()
        } else {
            let function = Rc::new(RefCell::new(RISCVValue::Function {
                name: name.to_string(),
                frame_size: 0,
            }));
            self.values.insert(key, function.clone());
            function
        };

        Ok(function)
    }

    fn decl_parameter(
        &mut self,
        name: &str,
        r#type: &Rc<RefCell<Type>>,
    ) -> Result<Self::Value, Diagnostic<usize>> {
        todo!()
    }

    fn decl_record(
        &mut self,
        name: &str,
        r#type: &Rc<RefCell<Type>>,
    ) -> Result<(), Diagnostic<usize>> {
        Ok(())
    }

    fn conditional_branch(
        &mut self,
        condition: &Self::Value,
        then_block: &Self::BasicBlock,
        else_block: &Self::BasicBlock,
    ) -> Result<(), Diagnostic<usize>> {
        self.add_instruction(
            "bneqz",
            &[
                condition.clone(),
                Rc::new(RefCell::new(RISCVValue::Symbol(
                    then_block.borrow().name.clone(),
                ))),
            ],
        )?;

        self.add_instruction(
            "beqz",
            &[
                condition.clone(),
                Rc::new(RefCell::new(RISCVValue::Symbol(
                    else_block.borrow().name.clone(),
                ))),
            ],
        )?;

        Ok(())
    }

    fn unconditional_branch(
        &mut self,
        dest_block: &Self::BasicBlock,
    ) -> Result<(), Diagnostic<usize>> {
        self.add_instruction(
            "j",
            &[Rc::new(RefCell::new(RISCVValue::Symbol(
                dest_block.borrow().name.clone(),
            )))],
        )?;

        Ok(())
    }

    fn switch(
        &mut self,
        condition: &Self::Value,
        condition_type: &Rc<RefCell<Type>>,
        cases: &[(Self::Value, Self::BasicBlock)],
        default: &Self::BasicBlock,
    ) -> Result<(), Diagnostic<usize>> {
        for (value, block) in cases {
            // 经过整数提升, 条件表达式的类型应该与case表达式的类型相同
            let t = self.binop(
                BinOpKind::Eq,
                condition,
                condition_type,
                value,
                condition_type,
            )?;
            self.add_instruction(
                "bneqz",
                &[
                    t.clone(),
                    Rc::new(RefCell::new(RISCVValue::Symbol(
                        block.borrow().name.clone(),
                    ))),
                ],
            )?;
        }
        self.unconditional_branch(default)?;
        Ok(())
    }

    fn r#return(&mut self, value: Option<Self::Value>) -> Result<(), Diagnostic<usize>> {
        //TODO 返回值
        self.add_instruction("ret", &[])?;

        Ok(())
    }

    fn load(
        &mut self,
        ptr: &Self::Value,
        r#type: &Rc<RefCell<Type>>,
    ) -> Result<Self::Value, Diagnostic<usize>> {
        let address = Rc::new(RefCell::new(RISCVValue::Address(ptr.clone(), 0)));
        match &get_inner_type(r#type.clone()).borrow().kind {
            t if t.is_integer() => {
                let is_unsigned = t.is_unsigned().unwrap();
                let mut size = t.size().unwrap();
                let mut rd = vec![];

                while size > 0 {
                    let t = self.assign_ireg();
                    let a = size.min(self.xlen);

                    match a {
                        8 => self.add_instruction(
                            if is_unsigned { "lbu" } else { "lb" },
                            &[t.clone(), address.clone()],
                        )?,
                        16 => self.add_instruction(
                            if is_unsigned { "lhu" } else { "lh" },
                            &[t.clone(), address.clone()],
                        )?,
                        32 => self.add_instruction(
                            if is_unsigned { "lwu" } else { "lw" },
                            &[t.clone(), address.clone()],
                        )?,
                        //如果 self.word_size < 64, 这个pattern不可能匹配到
                        64 => self.add_instruction(
                            if is_unsigned { "ldu" } else { "ld" },
                            &[t.clone(), address.clone()],
                        )?,
                        _ => {
                            self.add_instruction(
                                if a <= 8 {
                                    "lbu"
                                } else if a <= 16 {
                                    "lhu"
                                } else if a <= 32 {
                                    "lwu"
                                } else {
                                    "ldu"
                                },
                                &[t.clone(), address.clone()],
                            )?;
                            if !is_unsigned {
                                let shift = Rc::new(RefCell::new(RISCVValue::Integer(
                                    (self.xlen - a) as u64,
                                )));
                                self.add_instruction(
                                    "sll",
                                    &[t.clone(), t.clone(), shift.clone()],
                                )?;
                                self.add_instruction(
                                    "sra",
                                    &[t.clone(), t.clone(), shift.clone()],
                                )?;
                            }
                        }
                    }

                    rd.push(t);

                    if let RISCVValue::Address(_, offset) = &mut *address.borrow_mut() {
                        *offset += a as isize;
                    }

                    size -= self.xlen;
                }

                if rd.len() == 0 {
                    Ok(rd.remove(0))
                } else {
                    Ok(Rc::new(RefCell::new(RISCVValue::Array(rd))))
                }
            }
            TypeKind::Float => {
                let rd = self.assign_freg();
                self.add_instruction("flw", &[rd.clone(), address.clone()])?;
                Ok(rd)
            }
            TypeKind::Double => {
                let rd = self.assign_freg();
                self.add_instruction("fld", &[rd.clone(), address.clone()])?;
                Ok(rd)
            }
            //TODO long double
            //TODO 十进制浮点数
            TypeKind::Complex(Some(a)) => {
                let real = self.assign_freg();
                let imag = self.assign_freg();

                let real_address = address;
                let imag_address = Rc::new(RefCell::new(RISCVValue::Address(
                    ptr.clone(),
                    //TODO 可能需要考虑对齐
                    a.borrow().size().unwrap() as isize,
                )));

                if a.borrow().is_float() {
                    self.add_instruction("flw", &[real.clone(), real_address.clone()])?;
                    self.add_instruction("flw", &[imag.clone(), imag_address.clone()])?;
                } else if a.borrow().is_double() {
                    self.add_instruction("flw", &[real.clone(), real_address.clone()])?;
                    self.add_instruction("flw", &[imag.clone(), imag_address.clone()])?;
                } else if a.borrow().is_long_double() {
                    //TODO long double
                    todo!()
                } else {
                    unreachable!()
                }

                Ok(RISCVValue::new_complex(real, imag))
            }
            TypeKind::Pointer(_) => {
                let rd = self.assign_ireg();
                self.add_instruction(
                    if self.xlen == 32 {
                        "lwu"
                    } else if self.xlen == 64 {
                        "ldu"
                    } else {
                        unreachable!()
                    },
                    &[rd.clone(), address.clone()],
                )?;
                Ok(rd)
            }
            //TODO record
            _ => unreachable!(),
        }
    }

    fn load_var_ptr(&mut self, name: &str) -> Result<Self::Value, Diagnostic<usize>> {
        let symbol = self.lookup(name, Namespace::Ordinary).unwrap();

        let value = self
            .values
            .get(&(symbol.as_ptr() as usize))
            .unwrap()
            .clone();

        match &*value.borrow() {
            RISCVValue::Symbol(_) | RISCVValue::Function { .. } => {
                let rd = self.assign_ireg();
                self.add_instruction("la", &[rd.clone(), value.clone()])?;
                Ok(rd)
            }
            RISCVValue::Address(base, offset) => {
                let rd = self.assign_ireg();
                self.add_instruction(
                    "addi",
                    &[
                        rd.clone(),
                        base.clone(),
                        Rc::new(RefCell::new(RISCVValue::Integer(*offset as u64))),
                    ],
                )?;
                Ok(rd)
            }
            _ => unreachable!(),
        }
    }

    fn load_member_ptr(
        &mut self,
        target_value: &Self::Value,
        record_type: &Rc<RefCell<Type>>,
        member_name: &str,
    ) -> Result<Self::Value, Diagnostic<usize>> {
        match &record_type.borrow().kind {
            TypeKind::Record {
                kind: RecordKind::Struct,
                members: Some(members),
                ..
            } => {
                let mut offset = 0;
                for (name, symbol) in members {
                    if name == member_name {
                        break;
                    }
                    offset += symbol.borrow().r#type.borrow().size().unwrap();
                }

                let rd = self.assign_ireg();

                self.add_instruction(
                    "add",
                    &[
                        rd.clone(),
                        target_value.clone(),
                        Rc::new(RefCell::new(RISCVValue::Integer(offset as u64))),
                    ],
                )?;

                Ok(rd)
            }
            TypeKind::Record {
                kind: RecordKind::Union,
                ..
            } => Ok(target_value.clone()),
            _ => unreachable!(),
        }
    }

    fn load_bitfield(
        &mut self,
        value: &Self::Value,
        width: usize,
        offset: usize,
    ) -> Result<Self::Value, Diagnostic<usize>> {
        let rd = self.assign_ireg();
        let mask = Rc::new(RefCell::new(RISCVValue::Integer((1 << width) - 1)));

        self.add_instruction(
            "srl",
            &[
                rd.clone(),
                value.clone(),
                Rc::new(RefCell::new(RISCVValue::Integer(offset as u64))),
            ],
        )?;

        self.add_instruction("and", &[rd.clone(), rd.clone(), mask])?;

        Ok(rd)
    }

    fn load_compound_literal_ptr(
        &mut self,
        r#type: &Rc<RefCell<Type>>,
        storage_classes: &[StorageClass],
        init_value: &Self::Value,
    ) -> Result<Self::Value, Diagnostic<usize>> {
        let name = "compoundliteral";
        let decl_global = self.func_values.len() == 0
            || storage_classes
                .iter()
                .any(|x| x.kind == StorageClassKind::Static);
        if decl_global {
            self.globals
                .insert(name.to_string(), Some(init_value.clone()));
            let rd = self.assign_ireg();
            self.add_instruction(
                "la",
                &[
                    rd.clone(),
                    Rc::new(RefCell::new(RISCVValue::Symbol(name.to_string()))),
                ],
            )?;
            Ok(rd)
        } else {
            let offset = {
                let size = r#type.borrow().size().unwrap();
                let RISCVValue::Function { frame_size, .. } =
                    &mut *self.func_values.last_mut().unwrap().borrow_mut()
                else {
                    unreachable!();
                };
                let offset = *frame_size;
                *frame_size += size.next_power_of_two();
                offset
            };

            let rd = self.assign_ireg();
            self.add_instruction(
                "add",
                &[
                    rd.clone(),
                    Self::fp_reg(),
                    Rc::new(RefCell::new(RISCVValue::Integer(offset as u64))),
                ],
            )?;
            let address = Rc::new(RefCell::new(RISCVValue::Address(
                Self::fp_reg(),
                offset as isize,
            )));
            self.store(&address, &init_value, r#type)?;
            Ok(rd)
        }
    }

    fn store(
        &mut self,
        ptr: &Self::Value,
        value: &Self::Value,
        r#type: &Rc<RefCell<Type>>,
    ) -> Result<(), Diagnostic<usize>> {
        let address = Rc::new(RefCell::new(RISCVValue::Address(ptr.clone(), 0)));
        match &get_inner_type(r#type.clone()).borrow().kind {
            t if t.is_integer() => {
                let is_unsigned = t.is_unsigned().unwrap();
                let mut size = t.size().unwrap();

                while size > 0 {
                    let a = size.min(self.xlen);

                    match a {
                        8 => self.add_instruction("sb", &[value.clone(), address.clone()])?,
                        16 => self.add_instruction("sh", &[value.clone(), address.clone()])?,
                        32 => self.add_instruction("sw", &[value.clone(), address.clone()])?,
                        //如果 self.word_size < 64, 这个pattern不可能匹配到
                        64 => self.add_instruction("sd", &[value.clone(), address.clone()])?,
                        _ => {
                            if !is_unsigned {
                                let shift = Rc::new(RefCell::new(RISCVValue::Integer(
                                    (self.xlen - a) as u64,
                                )));
                                self.add_instruction(
                                    "sll",
                                    &[value.clone(), value.clone(), shift.clone()],
                                )?;
                                self.add_instruction(
                                    "sra",
                                    &[value.clone(), value.clone(), shift.clone()],
                                )?;
                            }
                            self.add_instruction(
                                if a <= 8 {
                                    "sb"
                                } else if a <= 16 {
                                    "sh"
                                } else if a <= 32 {
                                    "sw"
                                } else {
                                    "sd"
                                },
                                &[value.clone(), address.clone()],
                            )?;
                        }
                    }

                    if let RISCVValue::Address(_, offset) = &mut *address.borrow_mut() {
                        *offset += a as isize;
                    }

                    size -= self.xlen;
                }
            }
            TypeKind::Float => self.add_instruction("fsw", &[value.clone(), address.clone()])?,
            TypeKind::Double => self.add_instruction("fsd", &[value.clone(), address.clone()])?,
            //TODO long double
            //TODO 十进制浮点数
            TypeKind::Complex(Some(a)) => {
                let real;
                let imag;

                match &*value.borrow() {
                    RISCVValue::Struct(t) => {
                        real = t[0].clone();
                        imag = t[1].clone();
                    }
                    _ => unreachable!(),
                }

                let real_address = address;
                let imag_address = Rc::new(RefCell::new(RISCVValue::Address(
                    ptr.clone(),
                    //TODO 可能需要考虑对齐
                    a.borrow().size().unwrap() as isize,
                )));

                if a.borrow().is_float() {
                    self.add_instruction("fsw", &[real.clone(), real_address.clone()])?;
                    self.add_instruction("fsw", &[imag.clone(), imag_address.clone()])?;
                } else if a.borrow().is_double() {
                    self.add_instruction("fsd", &[real.clone(), real_address.clone()])?;
                    self.add_instruction("fsd", &[imag.clone(), imag_address.clone()])?;
                } else if a.borrow().is_long_double() {
                    //TODO long double
                    todo!()
                } else {
                    unreachable!()
                }
            }
            TypeKind::Pointer(_) => {
                self.add_instruction(
                    if self.xlen == 32 {
                        "sw"
                    } else if self.xlen == 64 {
                        "sd"
                    } else {
                        unreachable!()
                    },
                    &[value.clone(), address.clone()],
                )?;
            }
            TypeKind::Array { element_type, .. } => {
                let size = element_type.borrow().size().unwrap() as isize;
                let RISCVValue::Array(a) = &*value.borrow() else {
                    unreachable!()
                };
                for i in a {
                    self.store(&address, i, r#type)?;
                    if let RISCVValue::Address(_, offset) = &mut *address.borrow_mut() {
                        *offset += size;
                    }
                }
            }
            TypeKind::Record {
                kind: RecordKind::Struct,
                members: Some(members),
                ..
            } => {
                let RISCVValue::Struct(a) = &*value.borrow() else {
                    unreachable!()
                };
                for (i, v) in a.iter().enumerate() {
                    let member_type = members.get_index(i).unwrap().1.borrow().r#type.clone();
                    self.store(&address, v, &member_type)?;
                    let size = member_type.borrow().size().unwrap() as isize;
                    if let RISCVValue::Address(_, offset) = &mut *address.borrow_mut() {
                        *offset += size;
                    }
                }
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    fn store_bitfield(
        &mut self,
        ptr: &Self::Value,
        r#type: &Rc<RefCell<Type>>,
        value: &Self::Value,
        width: usize,
        offset: usize,
    ) -> Result<(), Diagnostic<usize>> {
        let old_value = self.load(ptr, r#type)?;

        let masked_value = self.assign_ireg();
        self.add_instruction(
            "and",
            &[
                masked_value.clone(),
                old_value.clone(),
                Rc::new(RefCell::new(RISCVValue::Integer(
                    !(((1 << width) - 1) << offset),
                ))),
            ],
        )?;

        let rd = self.assign_ireg();
        self.add_instruction(
            "and",
            &[
                rd.clone(),
                value.clone(),
                Rc::new(RefCell::new(RISCVValue::Integer((1 << width) - 1))),
            ],
        )?;
        self.add_instruction(
            "srl",
            &[
                rd.clone(),
                rd.clone(),
                Rc::new(RefCell::new(RISCVValue::Integer(offset as u64))),
            ],
        )?;
        self.add_instruction("or", &[rd.clone(), rd.clone(), masked_value.clone()])?;

        self.store(ptr, &rd, r#type)?;

        Ok(())
    }

    fn cast(
        &mut self,
        value: &Self::Value,
        value_type: &Rc<RefCell<Type>>,
        r#type: &Rc<RefCell<Type>>,
        method: CastMethod,
    ) -> Result<Self::Value, Diagnostic<usize>> {
        match method {
            CastMethod::Nothing
            | CastMethod::PtrToPtr
            | CastMethod::ArrayToPtr
            | CastMethod::FuncToPtr
            | CastMethod::PtrToInt => Ok(value.clone()),
            CastMethod::IntToPtr => match &*value.borrow() {
                //_BitInt
                RISCVValue::Array(a) => Ok(a[0].clone()),
                _ => Ok(value.clone()),
            },
            CastMethod::LToRValue => unreachable!(),
            CastMethod::ComplexExtend | CastMethod::ComplexTrunc => {
                let TypeKind::Complex(Some(value_part_type)) = &r#type.borrow().kind else {
                    unreachable!()
                };
                let TypeKind::Complex(Some(part_type)) = &r#type.borrow().kind else {
                    unreachable!()
                };
                let (real, imag) = self.extract_real_imag(value, value_type)?;

                let method = match method {
                    CastMethod::ComplexExtend => CastMethod::FloatExtend,
                    CastMethod::ComplexTrunc => CastMethod::FloatTrunc,
                    _ => unreachable!(),
                };

                let cast_real = self.cast(&real, value_part_type, part_type, method)?;
                let cast_imag = self.cast(&imag, value_part_type, part_type, method)?;

                Ok(RISCVValue::new_complex(cast_real, cast_imag))
            }
            CastMethod::ToBool => Ok(self.to_bool(value, value_type)?),
            CastMethod::FloatToComplex => {
                let TypeKind::Complex(Some(part_type)) = &r#type.borrow().kind else {
                    unreachable!()
                };
                let real = self.cast(
                    value,
                    value_type,
                    part_type,
                    if value_type.borrow().size().unwrap() < part_type.borrow().size().unwrap() {
                        CastMethod::FloatExtend
                    } else {
                        CastMethod::FloatTrunc
                    },
                )?;

                //TODO long double
                let t = self.load_float(&if value_type.borrow().is_float() {
                    RISCVValue::Float(0.)
                } else {
                    RISCVValue::Double(0.)
                })?;

                Ok(RISCVValue::new_complex(real, t))
            }
            CastMethod::ComplexToFloat => {
                let TypeKind::Complex(Some(part_type)) = &r#type.borrow().kind else {
                    unreachable!()
                };
                let (real, _) = self.extract_real_imag(value, value_type)?;
                Ok(self.cast(
                    &real,
                    part_type,
                    r#type,
                    if part_type.borrow().size().unwrap() < r#type.borrow().size().unwrap() {
                        CastMethod::FloatExtend
                    } else {
                        CastMethod::FloatTrunc
                    },
                )?)
            }
            CastMethod::FloatTrunc => {
                //TODO long double
                let rd = self.assign_freg();
                self.add_instruction("fcvt.d.s", &[rd.clone(), value.clone()])?;
                Ok(rd)
            }
            CastMethod::FloatExtend => {
                //TODO long double
                let rd = self.assign_freg();
                self.add_instruction("fcvt.s.d", &[rd.clone(), value.clone()])?;
                Ok(rd)
            }
            CastMethod::FloatToSInt | CastMethod::FloatToUInt => {
                //TODO long double
                let rd = self.assign_ireg();

                let mut opcode = "fcvt".to_string();

                //TODO _BitInt
                if r#type.borrow().size().unwrap() <= 32 {
                    opcode += ".w";
                } else {
                    opcode += ".l";
                }

                if let CastMethod::FloatToUInt = method {
                    opcode += "u";
                }

                if value_type.borrow().is_float() {
                    opcode += ".s"
                } else if value_type.borrow().is_double() {
                    opcode += ".d"
                } else {
                    unreachable!()
                }

                self.add_instruction(opcode, &[rd.clone(), value.clone()])?;

                Ok(rd)
            }
            CastMethod::SIntToFloat | CastMethod::UIntToFloat => {
                //TODO long double
                let rd = self.assign_freg();

                let mut opcode = "fcvt".to_string();

                if r#type.borrow().is_float() {
                    opcode += ".s"
                } else if r#type.borrow().is_double() {
                    opcode += ".d"
                } else {
                    unreachable!()
                }

                //TODO _BitInt
                if self.xlen <= 32 {
                    opcode += ".w";
                } else {
                    opcode += ".l";
                }

                if let CastMethod::UIntToFloat = method {
                    opcode += "u";
                }

                self.add_instruction(opcode, &[rd.clone(), value.clone()])?;

                Ok(rd)
            }
            CastMethod::SignedExtend => {
                let value_size = value_type.borrow().size().unwrap();
                let shift = Rc::new(RefCell::new(RISCVValue::Integer(
                    (self.xlen - value_size % self.xlen) as u64,
                )));

                let mut a = if let RISCVValue::Array(a) = &*value.borrow() {
                    a.clone()
                } else {
                    vec![value.clone()]
                };
                self.add_instruction("sll", &[a[a.len() - 1].clone(), shift.clone()])?;
                self.add_instruction("sra", &[a[a.len() - 1].clone(), shift.clone()])?;

                let mut value_size = value_size.next_power_of_two();
                let size: usize = r#type.borrow().size().unwrap().next_power_of_two();

                if value_size < size {
                    let sign_extend = self.assign_ireg();
                    self.add_instruction(
                        "sra",
                        &[
                            sign_extend.clone(),
                            a[a.len() - 1].clone(),
                            Rc::new(RefCell::new(RISCVValue::Integer((self.xlen - 1) as u64))),
                        ],
                    )?;

                    while value_size < size {
                        let t = self.assign_ireg();
                        self.add_instruction("mv", &[t.clone(), sign_extend.clone()])?;
                        a.push(t);
                        value_size += self.xlen;
                    }
                }

                if a.len() == 1 {
                    Ok(a.remove(0))
                } else {
                    Ok(Rc::new(RefCell::new(RISCVValue::Array(a))))
                }
            }
            CastMethod::ZeroExtand => {
                let value_size = value_type.borrow().size().unwrap();
                let shift = Rc::new(RefCell::new(RISCVValue::Integer(
                    (self.xlen - value_size % self.xlen) as u64,
                )));

                let mut a = if let RISCVValue::Array(a) = &*value.borrow() {
                    a.clone()
                } else {
                    vec![value.clone()]
                };
                self.add_instruction("sll", &[a[a.len() - 1].clone(), shift.clone()])?;
                self.add_instruction("srl", &[a[a.len() - 1].clone(), shift.clone()])?;

                let mut value_size = value_size.next_power_of_two();
                let size: usize = r#type.borrow().size().unwrap().next_power_of_two();

                if value_size < size {
                    while value_size < size {
                        a.push(Rc::new(RefCell::new(RISCVValue::Integer(0))));
                        value_size += self.xlen;
                    }
                }

                if a.len() == 1 {
                    Ok(a.remove(0))
                } else {
                    Ok(Rc::new(RefCell::new(RISCVValue::Array(a))))
                }
            }
            CastMethod::IntTrunc => match &*value.borrow() {
                RISCVValue::Array(a) => {
                    let mut a = a.clone();
                    let mut value_size = value_type.borrow().size().unwrap().next_power_of_two();
                    let size = r#type.borrow().size().unwrap();
                    while value_size > size.next_power_of_two() {
                        a.pop();
                        value_size -= self.xlen;
                    }

                    let shift = Rc::new(RefCell::new(RISCVValue::Integer(
                        (self.xlen - value_size % self.xlen) as u64,
                    )));
                    self.add_instruction("sll", &[a[a.len() - 1].clone(), shift.clone()])?;
                    self.add_instruction(
                        if r#type.borrow().is_unsigned().unwrap() {
                            "srl"
                        } else {
                            "sra"
                        },
                        &[a[a.len() - 1].clone(), shift.clone()],
                    )?;

                    if a.len() == 1 {
                        Ok(a.remove(0))
                    } else {
                        Ok(Rc::new(RefCell::new(RISCVValue::Array(a))))
                    }
                }
                _ => {
                    let rd = self.assign_ireg();
                    let size = r#type.borrow().size().unwrap();
                    let mask = Rc::new(RefCell::new(RISCVValue::Integer(((1 << size) - 1) as u64)));
                    self.add_instruction("and", &[rd.clone(), value.clone(), mask.clone()])?;
                    Ok(rd)
                }
            },
        }
    }

    fn subscript(
        &mut self,
        target: &Self::Value,
        index: &Self::Value,
        r#type: &Rc<RefCell<Type>>,
    ) -> Result<Self::Value, Diagnostic<usize>> {
        let pointee = pointee(r#type.clone()).unwrap();
        let rd = self.assign_ireg();
        let offset = self.assign_ireg();

        self.add_instruction(
            "mul",
            &[
                offset.clone(),
                index.clone(),
                Rc::new(RefCell::new(RISCVValue::Integer(
                    pointee.borrow().size().unwrap() as u64,
                ))),
            ],
        )?;

        self.add_instruction("add", &[rd.clone(), target.clone(), offset.clone()])?;

        Ok(rd)
    }

    fn call(
        &mut self,
        function: &Self::Value,
        args: &[Self::Value],
        r#type: &Rc<RefCell<Type>>,
    ) -> Result<Self::Value, Diagnostic<usize>> {
        let (args_location, retval_location) = self.call_convention(r#type)?;
        for (i, arg) in args.iter().enumerate() {
            let location = &args_location[i];
            match &*location.borrow() {
                RISCVValue::IReg(_) => match &*arg.borrow() {
                    RISCVValue::Integer(_) => {
                        self.add_instruction("li", &[location.clone(), arg.clone()])?;
                    }
                    RISCVValue::Float(t) => {
                        self.add_instruction(
                            "li",
                            &[
                                location.clone(),
                                Rc::new(RefCell::new(RISCVValue::Integer(t.to_bits() as u64))),
                            ],
                        )?;
                    }
                    //只有xlen>=64时才有可能匹配到这个pattern
                    RISCVValue::Double(t) => {
                        self.add_instruction(
                            "li",
                            &[
                                location.clone(),
                                Rc::new(RefCell::new(RISCVValue::Integer(t.to_bits() as u64))),
                            ],
                        )?;
                    }
                    RISCVValue::IReg(_) => {
                        self.add_instruction("mv", &[location.clone(), arg.clone()])?;
                    }
                    RISCVValue::FReg(_) => {
                        let TypeKind::Function {
                            parameters_type, ..
                        } = &r#type.borrow().kind
                        else {
                            unreachable!()
                        };
                        let arg_type = &parameters_type[i];
                        if arg_type.borrow().is_float() {
                            self.add_instruction("fmv.x.s", &[location.clone(), arg.clone()])?;
                        } else if arg_type.borrow().is_double() {
                            self.add_instruction("fmv.x.d", &[location.clone(), arg.clone()])?;
                        } else if arg_type.borrow().is_long_double() {
                            //TODO long double
                            todo!()
                        }
                    }
                    RISCVValue::Struct(_) => todo!(),
                    _ => unreachable!(),
                },
                RISCVValue::FReg(_) => match &*arg.borrow() {
                    RISCVValue::Float(t) => {
                        self.add_instruction(
                            "li",
                            &[
                                location.clone(),
                                Rc::new(RefCell::new(RISCVValue::Integer(t.to_bits() as u64))),
                            ],
                        )?;
                    }
                    RISCVValue::Double(t) => {
                        self.add_instruction(
                            "li",
                            &[
                                location.clone(),
                                Rc::new(RefCell::new(RISCVValue::Integer(t.to_bits() as u64))),
                            ],
                        )?;
                    }
                    RISCVValue::FReg(_) => {
                        let TypeKind::Function {
                            parameters_type, ..
                        } = &r#type.borrow().kind
                        else {
                            unreachable!()
                        };
                        let arg_type = &parameters_type[i];
                        if arg_type.borrow().is_float() {
                            self.add_instruction("fmv.x.s", &[location.clone(), arg.clone()])?;
                        } else if arg_type.borrow().is_double() {
                            self.add_instruction("fmv.x.d", &[location.clone(), arg.clone()])?;
                        } else if arg_type.borrow().is_long_double() {
                            //TODO long double
                            todo!()
                        }
                    }
                    _ => unreachable!(),
                },
                RISCVValue::Array(_) => match &*arg.borrow() {
                    RISCVValue::Array(_) => todo!(),
                    RISCVValue::Struct(_) => todo!(),
                    _ => unreachable!(),
                },
                RISCVValue::Address(..) => match &*arg.borrow() {
                    RISCVValue::Array(_) => todo!(),
                    RISCVValue::Struct(_) => todo!(),
                    _ => unreachable!(),
                },
                _ => unreachable!(),
            }
        }
        self.add_instruction("jalr", &[function.clone()])?;
        Ok(retval_location)
    }

    fn phi(
        &mut self,
        r#type: &Rc<RefCell<Type>>,
        incomings: &[(Self::Value, Self::BasicBlock)],
    ) -> Result<Self::Value, Diagnostic<usize>> {
        todo!()
    }

    fn unaryop(
        &mut self,
        op: UnaryOpKind,
        operand_value: &Self::Value,
        operand_type: &Rc<RefCell<Type>>,
    ) -> Result<Self::Value, Diagnostic<usize>> {
        match op {
            UnaryOpKind::AddressOf => Ok(operand_value.clone()),
            UnaryOpKind::Dereference => Ok(self.load(operand_value, operand_type)?),
            UnaryOpKind::Not => {
                let rd = self.assign_ireg();

                let t = self.to_bool(operand_value, operand_type)?;
                self.add_instruction("seqz", &[rd.clone(), t])?;

                Ok(rd)
            }
            UnaryOpKind::Positive => Ok(operand_value.clone()),
            UnaryOpKind::Negative => {
                match &operand_type.borrow().kind {
                    t if t.is_integer() => {
                        match &*operand_value.borrow() {
                            //超出单个寄存器长度的整数, 比如_BitInt
                            RISCVValue::Array(a) => {
                                let mut rd = vec![];
                                //进行整数提升后a和b长度一样
                                for _ in 0..a.len() {
                                    rd.push(self.assign_ireg());
                                }
                                for (i, v) in a.iter().enumerate() {
                                    self.add_instruction("not", &[rd[i].clone(), v.clone()])?;
                                }
                                Ok(self.build_int_add(
                                    &Rc::new(RefCell::new(RISCVValue::Array(rd))),
                                    &Rc::new(RefCell::new(RISCVValue::Integer(1))),
                                )?)
                            }
                            _ => {
                                let rd = self.assign_ireg();
                                self.add_instruction("neg", &[rd.clone(), operand_value.clone()])?;
                                Ok(rd)
                            }
                        }
                    }
                    t if t.is_float() => {
                        let rd = self.assign_freg();

                        self.add_instruction("fneg.w", &[rd.clone(), operand_value.clone()])?;

                        Ok(rd)
                    }
                    t if t.is_double() => {
                        let rd = self.assign_freg();

                        self.add_instruction("fneg.d", &[rd.clone(), operand_value.clone()])?;

                        Ok(rd)
                    }
                    t if t.is_long_double() => {
                        let rd = self.assign_freg();

                        self.add_instruction("fneg.q", &[rd.clone(), operand_value.clone()])?;

                        Ok(rd)
                    }
                    t if t.is_complex() => {
                        let complex_type = get_inner_type(operand_type.clone());
                        let TypeKind::Complex(Some(part_type)) = &complex_type.borrow().kind else {
                            unreachable!()
                        };

                        let (real, imag) = self.extract_real_imag(operand_value, operand_type)?;

                        let neg_real = self.unaryop(UnaryOpKind::Negative, &real, part_type)?;
                        let neg_imag = self.unaryop(UnaryOpKind::Negative, &imag, part_type)?;

                        Ok(RISCVValue::new_complex(neg_real, neg_imag))
                    }
                    //TODO 十进制浮点数
                    _ => todo!(),
                }
            }
            UnaryOpKind::BitNot => {
                match &*operand_value.borrow() {
                    //超出单个寄存器长度的整数, 比如_BitInt
                    RISCVValue::Array(a) => {
                        let mut rd = vec![];
                        //进行整数提升后a和b长度一样
                        for _ in 0..a.len() {
                            rd.push(self.assign_ireg());
                        }
                        for (i, v) in a.iter().enumerate() {
                            self.add_instruction("not", &[rd[i].clone(), v.clone()])?;
                        }
                        Ok(Rc::new(RefCell::new(RISCVValue::Array(rd))))
                    }
                    _ => {
                        let rd = self.assign_ireg();
                        self.add_instruction("not", &[rd.clone(), operand_value.clone()])?;
                        Ok(rd)
                    }
                }
            }
            UnaryOpKind::PostfixDec
            | UnaryOpKind::PrefixDec
            | UnaryOpKind::PostfixInc
            | UnaryOpKind::PrefixInc => {
                let offset = match op {
                    UnaryOpKind::PostfixDec | UnaryOpKind::PrefixDec => -1,
                    UnaryOpKind::PrefixInc | UnaryOpKind::PostfixInc => 1,
                    _ => unreachable!(),
                } as u64;
                //操作数是左值且没有进行过左值转换
                let old_value = self.load(operand_value, operand_type)?;

                let new_value = match &operand_type.borrow().kind {
                    t if t.is_integer() => self.build_int_add(
                        &old_value,
                        &Rc::new(RefCell::new(RISCVValue::Integer(offset))),
                    )?,
                    t if t.is_float() => {
                        let rd = self.assign_freg();
                        let t = self.load_float(&RISCVValue::Float(offset as f32))?;
                        self.add_instruction("fadd.s", &[rd.clone(), old_value.clone(), t])?;

                        rd
                    }
                    t if t.is_double() => {
                        let rd = self.assign_freg();
                        let t = self.load_float(&RISCVValue::Double(offset as f64))?;

                        self.add_instruction("fadd.w", &[rd.clone(), old_value.clone(), t])?;

                        rd
                    }
                    //TODO long double
                    t if t.is_long_double() => {
                        todo!()
                    }
                    t if t.is_pointer() => {
                        let rd = self.assign_ireg();
                        let pointee = pointee(operand_type.clone()).unwrap();

                        self.add_instruction(
                            "add",
                            &[
                                rd.clone(),
                                old_value.clone(),
                                Rc::new(RefCell::new(RISCVValue::Integer(
                                    offset * (pointee.borrow().size().unwrap() as u64),
                                ))),
                            ],
                        )?;

                        rd
                    }
                    _ => unreachable!(),
                };

                self.store(operand_value, &new_value, operand_type)?;

                match op {
                    UnaryOpKind::PostfixInc | UnaryOpKind::PostfixDec => Ok(old_value),
                    UnaryOpKind::PrefixInc | UnaryOpKind::PrefixDec => Ok(new_value),
                    _ => unreachable!(),
                }
            }
        }
    }

    fn binop(
        &mut self,
        op: BinOpKind,
        left_value: &Self::Value,
        left_type: &Rc<RefCell<Type>>,
        right_value: &Self::Value,
        right_type: &Rc<RefCell<Type>>,
    ) -> Result<Self::Value, Diagnostic<usize>> {
        match op {
            BinOpKind::Ge | BinOpKind::Gt | BinOpKind::Le | BinOpKind::Lt => {
                match (&left_type.borrow().kind, &right_type.borrow().kind) {
                    (a, b) if a.is_integer() && b.is_integer() => Ok(self.build_int_compare(
                        op,
                        left_value,
                        right_value,
                        a.is_unsigned().unwrap(),
                    )?),
                    (a, b) if a.is_float() && b.is_float() => {
                        let rd = self.assign_ireg();
                        let t1 = self.assign_freg();
                        match op {
                            BinOpKind::Lt => {
                                self.add_instruction(
                                    "flt.s",
                                    &[t1.clone(), left_value.clone(), right_value.clone()],
                                )?;
                            }
                            BinOpKind::Le => {
                                self.add_instruction(
                                    "fle.s",
                                    &[t1.clone(), left_value.clone(), right_value.clone()],
                                )?;
                            }
                            BinOpKind::Gt => {
                                self.add_instruction(
                                    "fle.s",
                                    &[t1.clone(), right_value.clone(), left_value.clone()],
                                )?;
                            }
                            BinOpKind::Ge => {
                                self.add_instruction(
                                    "fle.s",
                                    &[t1.clone(), right_value.clone(), left_value.clone()],
                                )?;
                            }
                            _ => unreachable!(),
                        }
                        self.add_instruction("fcvt.x.s", &[rd.clone(), t1])?;
                        Ok(rd)
                    }
                    (a, b) if a.is_double() && b.is_double() => {
                        let rd = self.assign_ireg();
                        let t1 = self.assign_freg();
                        match op {
                            BinOpKind::Lt => {
                                self.add_instruction(
                                    "flt.d",
                                    &[t1.clone(), left_value.clone(), right_value.clone()],
                                )?;
                            }
                            BinOpKind::Le => {
                                self.add_instruction(
                                    "fle.d",
                                    &[t1.clone(), left_value.clone(), right_value.clone()],
                                )?;
                            }
                            BinOpKind::Gt => {
                                self.add_instruction(
                                    "fle.d",
                                    &[t1.clone(), right_value.clone(), left_value.clone()],
                                )?;
                            }
                            BinOpKind::Ge => {
                                self.add_instruction(
                                    "fle.d",
                                    &[t1.clone(), right_value.clone(), left_value.clone()],
                                )?;
                            }
                            _ => unreachable!(),
                        }
                        self.add_instruction("fcvt.x.d", &[rd.clone(), t1])?;
                        Ok(rd)
                    }
                    (a, b) if a.is_long_double() && b.is_long_double() => {
                        let rd = self.assign_ireg();
                        let t1 = self.assign_freg();
                        match op {
                            BinOpKind::Lt => {
                                self.add_instruction(
                                    "flt.q",
                                    &[t1.clone(), left_value.clone(), right_value.clone()],
                                )?;
                            }
                            BinOpKind::Le => {
                                self.add_instruction(
                                    "fle.q",
                                    &[t1.clone(), left_value.clone(), right_value.clone()],
                                )?;
                            }
                            BinOpKind::Gt => {
                                self.add_instruction(
                                    "fle.q",
                                    &[t1.clone(), right_value.clone(), left_value.clone()],
                                )?;
                            }
                            BinOpKind::Ge => {
                                self.add_instruction(
                                    "fle.q",
                                    &[t1.clone(), right_value.clone(), left_value.clone()],
                                )?;
                            }
                            _ => unreachable!(),
                        }
                        self.add_instruction("fcvt.x.q", &[rd.clone(), t1])?;
                        Ok(rd)
                    }
                    (a, b) if a.is_pointer() && b.is_pointer() => {
                        let rd = self.assign_ireg();
                        self.add_instruction(
                            match op {
                                BinOpKind::Lt => "sltu",
                                BinOpKind::Le => "sleu",
                                BinOpKind::Gt => "sgtu",
                                BinOpKind::Ge => "sgeu",
                                _ => unreachable!(),
                            },
                            &[rd.clone(), left_value.clone(), right_value.clone()],
                        )?;
                        Ok(rd)
                    }
                    //TODO 十进制浮点数
                    _ => todo!(),
                }
            }
            BinOpKind::Eq | BinOpKind::Neq => {
                match (&left_type.borrow().kind, &right_type.borrow().kind) {
                    (a, b) if a.is_integer() && b.is_integer() => {
                        match (&*left_value.borrow(), &*right_value.borrow()) {
                            //超出单个寄存器长度的整数, 比如_BitInt
                            (RISCVValue::Array(x), RISCVValue::Array(y)) => {
                                let rd = self.assign_ireg();
                                let t = self.assign_ireg();
                                self.add_instruction(
                                    "li",
                                    &[rd.clone(), Rc::new(RefCell::new(RISCVValue::Integer(1)))],
                                )?;
                                for (x, y) in x.iter().zip(y) {
                                    self.add_instruction(
                                        "xor",
                                        &[t.clone(), x.clone(), y.clone()],
                                    )?;
                                    self.add_instruction("seqz", &[t.clone(), t.clone()])?;
                                    self.add_instruction("and", &[rd.clone(), t.clone()])?;
                                }
                                Ok(rd)
                            }
                            _ => {
                                let rd = self.assign_ireg();
                                self.add_instruction(
                                    "xor",
                                    &[rd.clone(), left_value.clone(), right_value.clone()],
                                )?;
                                self.add_instruction("seqz", &[rd.clone(), rd.clone()])?;
                                Ok(rd)
                            }
                        }
                    }
                    (a, b) if a.is_float() && b.is_float() => {
                        let rd = self.assign_ireg();
                        let t1 = self.assign_freg();

                        self.add_instruction(
                            "feq.s",
                            &[t1.clone(), left_value.clone(), right_value.clone()],
                        )?;
                        self.add_instruction("fcvt.x.s", &[rd.clone(), t1.clone()])?;

                        match op {
                            BinOpKind::Eq => {}
                            BinOpKind::Neq => {
                                self.add_instruction(
                                    "xori",
                                    &[
                                        rd.clone(),
                                        rd.clone(),
                                        Rc::new(RefCell::new(RISCVValue::Integer(1))),
                                    ],
                                )?;
                            }
                            _ => unreachable!(),
                        }
                        Ok(rd)
                    }
                    (a, b) if a.is_double() && b.is_double() => {
                        let rd = self.assign_ireg();
                        let t1 = self.assign_freg();

                        self.add_instruction(
                            "feq.d",
                            &[t1.clone(), left_value.clone(), right_value.clone()],
                        )?;
                        self.add_instruction("fcvt.x.d", &[rd.clone(), t1.clone()])?;

                        match op {
                            BinOpKind::Eq => {}
                            BinOpKind::Neq => {
                                self.add_instruction(
                                    "xori",
                                    &[
                                        rd.clone(),
                                        rd.clone(),
                                        Rc::new(RefCell::new(RISCVValue::Integer(1))),
                                    ],
                                )?;
                            }
                            _ => unreachable!(),
                        }
                        Ok(rd)
                    }
                    (a, b) if a.is_long_double() && b.is_long_double() => {
                        let rd = self.assign_ireg();
                        let t1 = self.assign_freg();

                        self.add_instruction(
                            "feq.q",
                            &[t1.clone(), left_value.clone(), right_value.clone()],
                        )?;
                        self.add_instruction("fcvt.x.q", &[rd.clone(), t1.clone()])?;

                        match op {
                            BinOpKind::Eq => {}
                            BinOpKind::Neq => {
                                self.add_instruction(
                                    "xori",
                                    &[
                                        rd.clone(),
                                        rd.clone(),
                                        Rc::new(RefCell::new(RISCVValue::Integer(1))),
                                    ],
                                )?;
                            }
                            _ => unreachable!(),
                        }
                        Ok(rd)
                    }
                    (a, b) if a.is_pointer() && b.is_pointer() => {
                        let rd = self.assign_ireg();
                        self.add_instruction(
                            match op {
                                BinOpKind::Eq => "sequ",
                                BinOpKind::Neq => "snequ",
                                _ => unreachable!(),
                            },
                            &[rd.clone(), left_value.clone(), right_value.clone()],
                        )?;
                        Ok(rd)
                    }
                    (a, b)
                        if (a.is_complex() && b.is_real_float())
                            || (b.is_complex() && a.is_real_float())
                            || (a.is_complex() && b.is_complex()) =>
                    {
                        let complex_type = if a.is_complex() {
                            get_inner_type(left_type.clone())
                        } else if b.is_complex() {
                            get_inner_type(right_type.clone())
                        } else {
                            unreachable!()
                        };
                        let TypeKind::Complex(Some(part_type)) = &complex_type.borrow().kind else {
                            unreachable!()
                        };

                        //经过隐式转换后, 如果其中一个为 xxx _Complex, 那么另一个类型一定为 xxx 或 xxx _Complex

                        let (real1, imag1) = self.extract_real_imag(left_value, left_type)?;
                        let (real2, imag2) = self.extract_real_imag(right_value, right_type)?;

                        let cmp_real = self.binop(op, &real1, part_type, &real2, part_type)?;
                        let cmp_imag = self.binop(op, &imag1, part_type, &imag2, part_type)?;

                        let rd = self.assign_ireg();
                        self.add_instruction(
                            "and",
                            &[rd.clone(), cmp_real.clone(), cmp_imag.clone()],
                        )?;

                        Ok(rd)
                    }
                    //TODO 十进制浮点数
                    _ => todo!(),
                }
            }
            BinOpKind::Add => {
                match (&left_type.borrow().kind, &right_type.borrow().kind) {
                    (a, b) if a.is_integer() && b.is_integer() => {
                        Ok(self.build_int_add(left_value, right_value)?)
                    }
                    (a, b) if a.is_float() && b.is_float() => {
                        let rd = self.assign_freg();
                        self.add_instruction(
                            "fadd.s",
                            &[rd.clone(), left_value.clone(), right_value.clone()],
                        )?;
                        Ok(rd)
                    }
                    (a, b) if a.is_double() && b.is_double() => {
                        let rd = self.assign_freg();
                        self.add_instruction(
                            "fadd.w",
                            &[rd.clone(), left_value.clone(), right_value.clone()],
                        )?;
                        Ok(rd)
                    }
                    (a, b) if a.is_long_double() && b.is_long_double() => {
                        let rd = self.assign_freg();
                        self.add_instruction(
                            "fadd.q",
                            &[rd.clone(), left_value.clone(), right_value.clone()],
                        )?;
                        Ok(rd)
                    }
                    (a, b) if a.is_pointer() && b.is_integer() => {
                        let rd = self.assign_ireg();
                        let offset = self.assign_ireg();
                        self.add_instruction(
                            "mul",
                            &[
                                offset.clone(),
                                right_value.clone(),
                                Rc::new(RefCell::new(
                                    RISCVValue::Integer(a.size().unwrap() as u64),
                                )),
                            ],
                        )?;
                        self.add_instruction(
                            "add",
                            &[rd.clone(), left_value.clone(), offset.clone()],
                        )?;
                        Ok(rd)
                    }
                    (a, b) if a.is_integer() && b.is_pointer() => {
                        let rd = self.assign_ireg();
                        let offset = self.assign_ireg();
                        self.add_instruction(
                            "mul",
                            &[
                                offset.clone(),
                                left_value.clone(),
                                Rc::new(RefCell::new(
                                    RISCVValue::Integer(a.size().unwrap() as u64),
                                )),
                            ],
                        )?;
                        self.add_instruction(
                            "add",
                            &[rd.clone(), right_value.clone(), offset.clone()],
                        )?;
                        Ok(rd)
                    }
                    (a, b)
                        if (a.is_complex() && b.is_real_float())
                            || (b.is_complex() && a.is_real_float())
                            || (a.is_complex() && b.is_complex()) =>
                    {
                        let complex_type = if a.is_complex() {
                            get_inner_type(left_type.clone())
                        } else if b.is_complex() {
                            get_inner_type(right_type.clone())
                        } else {
                            unreachable!()
                        };
                        let TypeKind::Complex(Some(part_type)) = &complex_type.borrow().kind else {
                            unreachable!()
                        };

                        //经过隐式转换后, 如果其中一个为 xxx _Complex, 那么另一个类型一定为 xxx 或 xxx _Complex

                        let (real1, imag1) = self.extract_real_imag(left_value, left_type)?;
                        let (real2, imag2) = self.extract_real_imag(right_value, right_type)?;

                        let new_real =
                            self.binop(BinOpKind::Add, &real1, part_type, &real2, part_type)?;
                        let new_imag =
                            self.binop(BinOpKind::Add, &imag1, part_type, &imag2, part_type)?;

                        Ok(RISCVValue::new_complex(new_real, new_imag))
                    }
                    //TODO 十进制浮点数
                    _ => todo!(),
                }
            }
            BinOpKind::Sub => {
                match (&left_type.borrow().kind, &right_type.borrow().kind) {
                    (a, b) if a.is_integer() && b.is_integer() => {
                        Ok(self.build_int_sub(left_value, right_value)?)
                    }
                    (a, b) if a.is_float() && b.is_float() => {
                        let rd = self.assign_freg();
                        self.add_instruction(
                            "fsub.s",
                            &[rd.clone(), left_value.clone(), right_value.clone()],
                        )?;
                        Ok(rd)
                    }
                    (a, b) if a.is_double() && b.is_double() => {
                        let rd = self.assign_freg();
                        self.add_instruction(
                            "fsub.w",
                            &[rd.clone(), left_value.clone(), right_value.clone()],
                        )?;
                        Ok(rd)
                    }
                    (a, b) if a.is_long_double() && b.is_long_double() => {
                        let rd = self.assign_freg();
                        self.add_instruction(
                            "fsub.q",
                            &[rd.clone(), left_value.clone(), right_value.clone()],
                        )?;
                        Ok(rd)
                    }
                    (a, b) if a.is_pointer() && b.is_integer() => {
                        let rd = self.assign_ireg();
                        let offset = self.assign_ireg();
                        self.add_instruction(
                            "mul",
                            &[
                                offset.clone(),
                                right_value.clone(),
                                Rc::new(RefCell::new(
                                    RISCVValue::Integer(a.size().unwrap() as u64),
                                )),
                            ],
                        )?;
                        self.add_instruction(
                            "sub",
                            &[rd.clone(), left_value.clone(), offset.clone()],
                        )?;
                        Ok(rd)
                    }
                    (a, b)
                        if (a.is_complex() && b.is_real_float())
                            || (b.is_complex() && a.is_real_float())
                            || (a.is_complex() && b.is_complex()) =>
                    {
                        let complex_type = if a.is_complex() {
                            get_inner_type(left_type.clone())
                        } else if b.is_complex() {
                            get_inner_type(right_type.clone())
                        } else {
                            unreachable!()
                        };
                        let TypeKind::Complex(Some(part_type)) = &complex_type.borrow().kind else {
                            unreachable!()
                        };

                        //经过隐式转换后, 如果其中一个为 xxx _Complex, 那么另一个类型一定为 xxx 或 xxx _Complex

                        let (real1, imag1) = self.extract_real_imag(left_value, left_type)?;
                        let (real2, imag2) = self.extract_real_imag(right_value, right_type)?;

                        let new_real =
                            self.binop(BinOpKind::Sub, &real1, part_type, &real2, part_type)?;
                        let new_imag =
                            self.binop(BinOpKind::Sub, &imag1, part_type, &imag2, part_type)?;

                        Ok(RISCVValue::new_complex(new_real, new_imag))
                    }
                    //TODO 十进制浮点数
                    _ => todo!(),
                }
            }
            BinOpKind::Mul | BinOpKind::Div | BinOpKind::Mod => {
                match (&left_type.borrow().kind, &right_type.borrow().kind) {
                    (a, b) if a.is_integer() && b.is_integer() => {
                        let opcode = match op {
                            BinOpKind::Mul => {
                                return Ok(self.build_int_mul(
                                    left_value,
                                    right_value,
                                    a.is_unsigned().unwrap(),
                                )?);
                            }
                            //TODO 调用函数实现
                            BinOpKind::Div => {
                                if a.is_unsigned().unwrap() {
                                    "divu"
                                } else {
                                    "div"
                                }
                            }
                            BinOpKind::Mod => {
                                if a.is_unsigned().unwrap() {
                                    "remu"
                                } else {
                                    "rem"
                                }
                            }
                            _ => unreachable!(),
                        };
                        let rd = self.assign_ireg();
                        self.add_instruction(
                            opcode,
                            &[rd.clone(), left_value.clone(), right_value.clone()],
                        )?;
                        Ok(rd)
                    }
                    (a, b) if a.is_float() && b.is_float() => {
                        let rd = self.assign_freg();
                        self.add_instruction(
                            match op {
                                BinOpKind::Mul => "fmul.s",
                                BinOpKind::Div => "fdiv.s",
                                _ => unreachable!(),
                            },
                            &[rd.clone(), left_value.clone(), right_value.clone()],
                        )?;
                        Ok(rd)
                    }
                    (a, b) if a.is_double() && b.is_double() => {
                        let rd = self.assign_freg();
                        self.add_instruction(
                            match op {
                                BinOpKind::Mul => "fmul.w",
                                BinOpKind::Div => "fdiv.w",
                                _ => unreachable!(),
                            },
                            &[rd.clone(), left_value.clone(), right_value.clone()],
                        )?;
                        Ok(rd)
                    }
                    (a, b) if a.is_long_double() && b.is_long_double() => {
                        let rd = self.assign_freg();
                        self.add_instruction(
                            match op {
                                BinOpKind::Mul => "fmul.q",
                                BinOpKind::Div => "fdiv.q",
                                _ => unreachable!(),
                            },
                            &[rd.clone(), left_value.clone(), right_value.clone()],
                        )?;
                        Ok(rd)
                    }
                    (a, b)
                        if (a.is_complex() && b.is_real_float())
                            || (b.is_complex() && a.is_real_float())
                            || (a.is_complex() && b.is_complex()) =>
                    {
                        let complex_type = if a.is_complex() {
                            get_inner_type(left_type.clone())
                        } else if b.is_complex() {
                            get_inner_type(right_type.clone())
                        } else {
                            unreachable!()
                        };
                        let TypeKind::Complex(Some(part_type)) = &complex_type.borrow().kind else {
                            unreachable!()
                        };

                        //经过隐式转换后, 如果其中一个为 xxx _Complex, 那么另一个类型一定为 xxx 或 xxx _Complex

                        let (real1, imag1) = self.extract_real_imag(left_value, left_type)?;
                        let (real2, imag2) = self.extract_real_imag(right_value, right_type)?;

                        let new_real = match op {
                            BinOpKind::Mul => {
                                let t1 = self.binop(
                                    BinOpKind::Mul,
                                    &real1,
                                    part_type,
                                    &real2,
                                    part_type,
                                )?;
                                let t2 = self.binop(
                                    BinOpKind::Mul,
                                    &imag1,
                                    part_type,
                                    &imag2,
                                    part_type,
                                )?;
                                self.binop(BinOpKind::Sub, &t1, part_type, &t2, part_type)?
                            }
                            BinOpKind::Div => {
                                let t1 = self.binop(
                                    BinOpKind::Mul,
                                    &real1,
                                    part_type,
                                    &real2,
                                    part_type,
                                )?;
                                let t2 = self.binop(
                                    BinOpKind::Mul,
                                    &imag1,
                                    part_type,
                                    &imag2,
                                    part_type,
                                )?;

                                let t3 = self.binop(
                                    BinOpKind::Mul,
                                    &real2,
                                    part_type,
                                    &real2,
                                    part_type,
                                )?;
                                let t4 = self.binop(
                                    BinOpKind::Mul,
                                    &imag2,
                                    part_type,
                                    &imag2,
                                    part_type,
                                )?;

                                let t5 =
                                    self.binop(BinOpKind::Add, &t1, part_type, &t2, part_type)?;
                                let t6 =
                                    self.binop(BinOpKind::Add, &t3, part_type, &t4, part_type)?;

                                self.binop(BinOpKind::Div, &t5, part_type, &t6, part_type)?
                            }
                            _ => unreachable!(),
                        };

                        let new_imag = match op {
                            BinOpKind::Mul => {
                                let t1 = self.binop(
                                    BinOpKind::Mul,
                                    &real1,
                                    part_type,
                                    &real2,
                                    part_type,
                                )?;
                                let t2 = self.binop(
                                    BinOpKind::Mul,
                                    &real2,
                                    part_type,
                                    &imag1,
                                    part_type,
                                )?;
                                self.binop(BinOpKind::Add, &t1, part_type, &t2, part_type)?
                            }
                            BinOpKind::Div => {
                                let t1 = self.binop(
                                    BinOpKind::Mul,
                                    &imag1,
                                    part_type,
                                    &real2,
                                    part_type,
                                )?;
                                let t2 = self.binop(
                                    BinOpKind::Mul,
                                    &real1,
                                    part_type,
                                    &imag2,
                                    part_type,
                                )?;

                                let t3 = self.binop(
                                    BinOpKind::Mul,
                                    &real2,
                                    part_type,
                                    &real2,
                                    part_type,
                                )?;
                                let t4 = self.binop(
                                    BinOpKind::Mul,
                                    &imag2,
                                    part_type,
                                    &imag2,
                                    part_type,
                                )?;

                                let t5 =
                                    self.binop(BinOpKind::Sub, &t1, part_type, &t2, part_type)?;
                                let t6 =
                                    self.binop(BinOpKind::Add, &t3, part_type, &t4, part_type)?;

                                self.binop(BinOpKind::Div, &t5, part_type, &t6, part_type)?
                            }
                            _ => unreachable!(),
                        };

                        Ok(RISCVValue::new_complex(new_real, new_imag))
                    }
                    //TODO 十进制浮点数
                    _ => todo!(),
                }
            }
            BinOpKind::BitAnd => Ok(self.build_int_bitand(left_value, right_value)?),
            BinOpKind::BitOr => Ok(self.build_int_bitor(left_value, right_value)?),
            BinOpKind::LShift => Ok(self.build_int_left_shift(left_value, right_value)?),
            BinOpKind::BitXOr | BinOpKind::RShift => {
                match (&left_type.borrow().kind, &right_type.borrow().kind) {
                    (a, b) if a.is_integer() && b.is_integer() => {
                        let opcode = match op {
                            BinOpKind::BitAnd => "and",
                            BinOpKind::BitOr => "or",
                            BinOpKind::BitXOr => "xor",
                            BinOpKind::LShift => "sll",
                            BinOpKind::RShift => "srl",
                            _ => unreachable!(),
                        };
                        match (&*left_value.borrow(), &*right_value.borrow()) {
                            //超出单个寄存器长度的整数, 比如_BitInt
                            (RISCVValue::Array(x), RISCVValue::Array(y)) => {
                                let mut rd = vec![];
                                //进行整数提升后a和b长度一样
                                for _ in 0..x.len() {
                                    rd.push(self.assign_ireg());
                                }
                                for (i, (x, y)) in x.iter().zip(y).enumerate() {
                                    self.add_instruction(
                                        if opcode == "srl"
                                            && i == rd.len() - 1
                                            && !a.is_unsigned().unwrap()
                                        {
                                            "sra"
                                        } else {
                                            opcode
                                        },
                                        &[rd[i].clone(), x.clone(), y.clone()],
                                    )?;
                                }
                                Ok(Rc::new(RefCell::new(RISCVValue::Array(rd))))
                            }
                            _ => {
                                let rd = self.assign_ireg();
                                self.add_instruction(
                                    if opcode == "srl" && !a.is_unsigned().unwrap() {
                                        "sra"
                                    } else {
                                        opcode
                                    },
                                    &[rd.clone(), left_value.clone(), right_value.clone()],
                                )?;
                                Ok(rd)
                            }
                        }
                    }
                    _ => unreachable!(),
                }
            }
            BinOpKind::Comma => Ok(right_value.clone()),
            _ => todo!(),
        }
    }
}
