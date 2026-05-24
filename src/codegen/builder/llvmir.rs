use crate::{
    ast::{
        decl::{FunctionSpecKind, StorageClassKind},
        expr::{BinOpKind, CastMethod, UnaryOpKind},
    },
    codegen::builder::Builder,
    ctype::{
        RecordKind, Type, TypeKind, array_element,
        layout::{ConstDesignation, Layout, compute_layout},
        pointee,
    },
    symtab::{Namespace, Symbol, SymbolKind, SymbolTable},
    variant::{Variant, to_decimal},
};
use codespan::Span;
use codespan_reporting::diagnostic::{Diagnostic, Label};
use inkwell::{
    AddressSpace, FloatPredicate, IntPredicate,
    basic_block::BasicBlock,
    builder::{self, BuilderError},
    context::Context,
    module::{Linkage, Module},
    types::{AnyType, AnyTypeEnum, BasicType, BasicTypeEnum, StringRadix},
    values::{
        AnyValue, AnyValueEnum, BasicValue, BasicValueEnum, FloatValue, FunctionValue,
        InstructionOpcode, IntValue,
    },
};
use num::{BigInt, BigRational, ToPrimitive, Zero};
use std::{cell::RefCell, collections::HashMap, fmt::Display, num::NonZero, rc::Rc};

pub struct LLVMIRBuilder<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: builder::Builder<'ctx>,
    pub symtabs: Vec<Rc<RefCell<SymbolTable>>>,
    pub func_values: Vec<FunctionValue<'ctx>>,
    pub file_ids: Vec<usize>,
    pub spans: Vec<Span>,
    pub types: HashMap<usize, AnyTypeEnum<'ctx>>,
    pub values: HashMap<usize, AnyValueEnum<'ctx>>,
}

impl<'ctx> LLVMIRBuilder<'ctx> {
    pub fn new(context: &'ctx Context, module: Module<'ctx>) -> LLVMIRBuilder<'ctx> {
        LLVMIRBuilder {
            context,
            module,
            builder: context.create_builder(),
            symtabs: vec![],
            func_values: vec![],
            file_ids: vec![],
            spans: vec![],
            types: HashMap::new(),
            values: HashMap::new(),
        }
    }

    pub fn error(&self, message: impl Display) -> Diagnostic<usize> {
        Diagnostic::error()
            .with_message(message)
            .with_label(Label::primary(
                *self.file_ids.last().unwrap(),
                *self.spans.last().unwrap(),
            ))
    }

    pub fn map_builder_err<T>(
        &self,
        result: Result<T, BuilderError>,
    ) -> Result<T, Diagnostic<usize>> {
        match result {
            Ok(t) => Ok(t),
            Err(e) => Err(self.error(format!("{e}"))).unwrap(),
        }
    }

    pub fn map_builder_err_with<T>(
        &self,
        result: Result<T, BuilderError>,
        file_id: usize,
        span: Span,
    ) -> Result<T, Diagnostic<usize>> {
        match result {
            Ok(t) => Ok(t),
            Err(e) => Err(Diagnostic::error()
                .with_message(e)
                .with_label(Label::primary(file_id, span))),
        }
    }

    pub fn to_bool(&self, value: AnyValueEnum<'ctx>) -> Result<IntValue<'ctx>, Diagnostic<usize>> {
        match value {
            AnyValueEnum::IntValue(t) => {
                Ok(self.map_builder_err(self.builder.build_int_compare(
                    IntPredicate::NE,
                    t,
                    t.get_type().const_int(0, false),
                    "",
                ))?)
            }
            AnyValueEnum::FloatValue(t) => {
                Ok(self.map_builder_err(self.builder.build_float_compare(
                    FloatPredicate::ONE,
                    t,
                    t.get_type().const_float(0.),
                    "",
                ))?)
            }
            AnyValueEnum::PointerValue(t) => Ok(self.map_builder_err(
                self.builder
                    .build_int_compare(IntPredicate::NE, t, t.get_type().const_null(), ""),
            )?),
            AnyValueEnum::StructValue(t) => {
                let mut a = self.context.i8_type().const_int(1, false);
                for field in t.get_fields() {
                    a = self.map_builder_err(self.builder.build_and(
                        a,
                        self.to_bool(field.as_any_value_enum())?,
                        "",
                    ))?;
                }
                Ok(a)
            }
            //TODO 十进制浮点数
            _ => todo!(),
        }
    }

    pub fn extract_real_imag(
        &self,
        a: AnyValueEnum<'ctx>,
    ) -> Result<(FloatValue<'ctx>, FloatValue<'ctx>), BuilderError> {
        match a {
            AnyValueEnum::FloatValue(t) => Ok((t, t.get_type().const_float(0.))),
            AnyValueEnum::StructValue(t) => Ok((
                self.builder
                    .build_extract_value(t, 0, "real")?
                    .into_float_value(),
                self.builder
                    .build_extract_value(t, 1, "imag")?
                    .into_float_value(),
            )),
            _ => Ok((
                self.context.f64_type().const_float(0.),
                self.context.f64_type().const_float(0.),
            )),
        }
    }

    pub fn to_llvm_type(
        &self,
        r#type: &Rc<RefCell<Type>>,
    ) -> Result<AnyTypeEnum<'ctx>, Diagnostic<usize>> {
        Ok(match &r#type.borrow().kind {
            TypeKind::Void => self.context.void_type().as_any_type_enum(),
            TypeKind::Bool => self.context.bool_type().as_any_type_enum(),
            TypeKind::Float => self.context.f32_type().as_any_type_enum(),
            TypeKind::Double => self.context.f64_type().as_any_type_enum(),
            TypeKind::LongDouble => self.context.f128_type().as_any_type_enum(),
            TypeKind::Complex(r#type) => {
                let t = if let Some(t) = r#type {
                    match self.to_llvm_type(t)? {
                        AnyTypeEnum::FloatType(t) => t.as_basic_type_enum(),
                        _ => self.context.f64_type().as_basic_type_enum(),
                    }
                } else {
                    self.context.f64_type().as_basic_type_enum()
                };
                self.context.struct_type(&[t, t], false).as_any_type_enum()
            }
            //TODO 十进制浮点数
            TypeKind::Decimal32 => todo!(),
            TypeKind::Decimal64 => todo!(),
            TypeKind::Decimal128 => todo!(),
            TypeKind::Pointer(..) | TypeKind::Nullptr => self
                .context
                .ptr_type(AddressSpace::default())
                .as_any_type_enum(),
            TypeKind::Qualified { r#type, .. }
            | TypeKind::Atomic(r#type)
            | TypeKind::Typedef {
                r#type: Some(r#type),
                ..
            }
            | TypeKind::Typeof {
                expr: None,
                r#type: Some(r#type),
                ..
            }
            | TypeKind::Auto(Some(r#type)) => self.to_llvm_type(r#type)?,
            TypeKind::Typedef { r#type: None, .. }
            | TypeKind::Typeof {
                expr: None,
                r#type: None,
                ..
            }
            | TypeKind::Auto(None) => {
                return Err(Diagnostic::error()
                    .with_message("incomplete type")
                    .with_label(Label::primary(
                        r#type.borrow().file_id,
                        r#type.borrow().span,
                    )));
            }
            TypeKind::Typeof {
                expr: Some(expr),
                r#type: None,
                ..
            } => self.to_llvm_type(&expr.borrow().r#type)?,
            TypeKind::Array {
                element_type,
                len_expr: Some(len_expr),
                ..
            } => {
                let size = match &len_expr.borrow().value {
                    Variant::Int(value) => match value.to_u32() {
                        Some(t) => t,
                        None => {
                            //跟TypeChecker::check_type完全一样
                            return Err(Diagnostic::error()
                                .with_message(format!("invalid integer for array length"))
                                .with_label(Label::primary(
                                    len_expr.borrow().file_id,
                                    len_expr.borrow().span,
                                )));
                        }
                    },
                    t if t.is_unknown() => {
                        return Ok(self
                            .context
                            .ptr_type(AddressSpace::default())
                            .as_any_type_enum());
                    }
                    _ => {
                        return Err(Diagnostic::error()
                            .with_message(format!("array length must be an integer constant"))
                            .with_label(Label::primary(
                                len_expr.borrow().file_id,
                                len_expr.borrow().span,
                            )));
                    }
                };
                let llvm_elem_ty = self.to_llvm_type(element_type)?;
                match llvm_elem_ty {
                    AnyTypeEnum::ArrayType(_) => llvm_elem_ty
                        .into_array_type()
                        .array_type(size)
                        .as_any_type_enum(),
                    AnyTypeEnum::FloatType(_) => llvm_elem_ty
                        .into_float_type()
                        .array_type(size)
                        .as_any_type_enum(),
                    AnyTypeEnum::IntType(_) => llvm_elem_ty
                        .into_int_type()
                        .array_type(size)
                        .as_any_type_enum(),
                    AnyTypeEnum::PointerType(_) => llvm_elem_ty
                        .into_pointer_type()
                        .array_type(size)
                        .as_any_type_enum(),
                    AnyTypeEnum::ScalableVectorType(_) => llvm_elem_ty
                        .into_scalable_vector_type()
                        .array_type(size)
                        .as_any_type_enum(),
                    AnyTypeEnum::StructType(_) => llvm_elem_ty
                        .into_struct_type()
                        .array_type(size)
                        .as_any_type_enum(),
                    AnyTypeEnum::VectorType(_) => llvm_elem_ty
                        .into_vector_type()
                        .array_type(size)
                        .as_any_type_enum(),
                    _ => {
                        return Err(Diagnostic::error()
                            .with_message(format!(
                                "invalid element type for array type: {}",
                                element_type.borrow()
                            ))
                            .with_label(Label::primary(
                                element_type.borrow().file_id,
                                element_type.borrow().span,
                            )));
                    }
                }
            }
            TypeKind::Array { len_expr: None, .. } => {
                return Err(Diagnostic::error()
                    .with_message(format!("unknown length of array type"))
                    .with_label(Label::primary(
                        r#type.borrow().file_id,
                        r#type.borrow().span,
                    )));
            }
            TypeKind::Function {
                return_type,
                parameters_type,
                has_varparam,
            } => {
                let mut param_types = vec![];
                for parameter_type in parameters_type {
                    let llvm_param_ty = self.to_llvm_type(parameter_type)?;

                    param_types.push(match llvm_param_ty {
                        AnyTypeEnum::ArrayType(_) => llvm_param_ty.into_array_type().into(),
                        AnyTypeEnum::FloatType(_) => llvm_param_ty.into_float_type().into(),
                        AnyTypeEnum::IntType(_) => llvm_param_ty.into_int_type().into(),
                        AnyTypeEnum::PointerType(_) => llvm_param_ty.into_pointer_type().into(),
                        AnyTypeEnum::ScalableVectorType(_) => {
                            llvm_param_ty.into_scalable_vector_type().into()
                        }
                        AnyTypeEnum::StructType(_) => llvm_param_ty.into_struct_type().into(),
                        AnyTypeEnum::VectorType(_) => llvm_param_ty.into_vector_type().into(),
                        AnyTypeEnum::VoidType(_) => break,
                        _ => {
                            return Err(Diagnostic::error()
                                .with_message(format!(
                                    "invalid type as parameter type: {}",
                                    parameter_type.borrow()
                                ))
                                .with_label(Label::primary(
                                    parameter_type.borrow().file_id,
                                    parameter_type.borrow().span,
                                )));
                        }
                    });
                }
                let llvm_ret_ty = self.to_llvm_type(return_type)?;
                match llvm_ret_ty {
                    AnyTypeEnum::ArrayType(_) => llvm_ret_ty
                        .into_array_type()
                        .fn_type(&param_types, *has_varparam)
                        .as_any_type_enum(),
                    AnyTypeEnum::FloatType(_) => llvm_ret_ty
                        .into_float_type()
                        .fn_type(&param_types, *has_varparam)
                        .as_any_type_enum(),
                    AnyTypeEnum::IntType(_) => llvm_ret_ty
                        .into_int_type()
                        .fn_type(&param_types, *has_varparam)
                        .as_any_type_enum(),
                    AnyTypeEnum::PointerType(_) => llvm_ret_ty
                        .into_pointer_type()
                        .fn_type(&param_types, *has_varparam)
                        .as_any_type_enum(),
                    AnyTypeEnum::ScalableVectorType(_) => llvm_ret_ty
                        .into_scalable_vector_type()
                        .fn_type(&param_types, *has_varparam)
                        .as_any_type_enum(),
                    AnyTypeEnum::StructType(_) => llvm_ret_ty
                        .into_struct_type()
                        .fn_type(&param_types, *has_varparam)
                        .as_any_type_enum(),
                    AnyTypeEnum::VoidType(_) => llvm_ret_ty
                        .into_void_type()
                        .fn_type(&param_types, *has_varparam)
                        .as_any_type_enum(),
                    AnyTypeEnum::VectorType(_) => llvm_ret_ty
                        .into_vector_type()
                        .fn_type(&param_types, *has_varparam)
                        .as_any_type_enum(),
                    _ => {
                        return Err(Diagnostic::error()
                            .with_message(format!(
                                "invalid return type for function type: {}",
                                return_type.borrow()
                            ))
                            .with_label(Label::primary(
                                return_type.borrow().file_id,
                                return_type.borrow().span,
                            )));
                    }
                }
            }
            TypeKind::Record { name, .. } => *self
                .types
                .get(&(self.lookup(name, Namespace::Tag).unwrap().as_ptr() as usize))
                .unwrap(),
            t if t.is_integer() => match r#type.borrow().size() {
                Some(1) => self.context.i8_type().as_any_type_enum(),
                Some(2) => self.context.i16_type().as_any_type_enum(),
                Some(4) => self.context.i32_type().as_any_type_enum(),
                Some(8) => self.context.i64_type().as_any_type_enum(),
                Some(16) => self.context.i128_type().as_any_type_enum(),
                Some(t) => {
                    self.context
                        //根据语义, BitInt的长度至少为1
                        .custom_width_int_type(NonZero::new((t * 8) as u32).unwrap())
                        .map_err(|x| {
                            Diagnostic::error().with_message(x.to_string()).with_label(
                                Label::primary(r#type.borrow().file_id, r#type.borrow().span),
                            )
                        })?
                        .as_any_type_enum()
                }
                None => {
                    return Err(Diagnostic::error()
                        .with_message("unknown size of integer type")
                        .with_label(Label::primary(
                            r#type.borrow().file_id,
                            r#type.borrow().span,
                        )));
                }
            },
            _ => unreachable!(),
        })
    }
}

impl<'ctx> Builder for LLVMIRBuilder<'ctx> {
    type Value = AnyValueEnum<'ctx>;
    type BasicBlock = BasicBlock<'ctx>;

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

    fn lookup(&self, name: &str, namespace: Namespace) -> Option<Rc<RefCell<Symbol>>> {
        self.symtabs
            .last()
            .unwrap()
            .borrow()
            .lookup(namespace, name)
    }

    fn enter_function(&mut self, function: &Self::Value) {
        let function = function.into_function_value();

        self.func_values.push(function);

        let basic_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(basic_block);
    }

    fn leave_function(&mut self, function: &Self::Value) -> Result<(), Diagnostic<usize>> {
        let function = function.into_function_value();
        self.func_values.pop();

        let mut last_block = function.get_first_basic_block().unwrap();
        while let Some(t) = last_block.get_next_basic_block() {
            if t.get_name().to_str().unwrap().starts_with("unreach") {
                unsafe { t.delete() }.unwrap();
                continue;
            }
            last_block = t;
        }
        if let None = last_block.get_terminator() {
            if let Some(t) = function.get_type().get_return_type() {
                let retval = self.map_builder_err(self.builder.build_alloca(t, ""))?;

                self.map_builder_err(self.builder.build_memset(
                    retval,
                    1,
                    self.context.i8_type().const_int(0, false),
                    t.size_of().unwrap(),
                ))?;

                self.map_builder_err(self.builder.build_return(Some(&retval)))?;
            } else {
                self.map_builder_err(self.builder.build_return(None))?;
            }
        }
        Ok(())
    }

    fn append_basic_block(&mut self, name: &str) -> Result<Self::BasicBlock, Diagnostic<usize>> {
        let block = self
            .context
            .append_basic_block(*self.func_values.last().unwrap(), name);
        block.move_after(self.current_basic_block()?).unwrap();
        Ok(block)
    }

    fn current_basic_block(&self) -> Result<Self::BasicBlock, Diagnostic<usize>> {
        Ok(self.builder.get_insert_block().unwrap())
    }

    fn position_at_end(&self, basic_block: &Self::BasicBlock) {
        self.builder.position_at_end(*basic_block);
    }

    fn get_previous_block(&self, basic_block: &Self::BasicBlock) -> Option<Self::BasicBlock> {
        basic_block.get_previous_basic_block()
    }

    fn variant_to_value(
        &self,
        variant: &Variant,
        r#type: &Rc<RefCell<Type>>,
    ) -> Result<Self::Value, Diagnostic<usize>> {
        match &variant {
            Variant::Bool(value) => Ok(match self.to_llvm_type(r#type)? {
                AnyTypeEnum::IntType(t) => t.const_int(*value as u64, false).as_any_value_enum(),
                AnyTypeEnum::FloatType(t) => unsafe {
                    t.const_float_from_string(&value.to_string())
                        .as_any_value_enum()
                },
                //可能是复数类型
                AnyTypeEnum::StructType(_) => self.variant_to_value(
                    &Variant::Complex(
                        BigRational::from_integer(BigInt::from(*value)),
                        BigRational::zero(),
                    ),
                    r#type,
                )?,
                _ => self
                    .context
                    .bool_type()
                    .const_int(*value as u64, false)
                    .as_any_value_enum(),
            }),
            Variant::Rational(value) => {
                let value_str = to_decimal(value).to_string();
                let dot_index = value_str.find(".").unwrap_or(value_str.len());
                Ok(match self.to_llvm_type(r#type)? {
                    AnyTypeEnum::FloatType(t) => {
                        unsafe { t.const_float_from_string(&value_str) }.as_any_value_enum()
                    }
                    AnyTypeEnum::IntType(t) => t
                        .const_int_from_string(&value_str[..dot_index], StringRadix::Decimal)
                        .unwrap()
                        .as_any_value_enum(),
                    //可能是复数类型
                    AnyTypeEnum::StructType(_) => self.variant_to_value(
                        &Variant::Complex(value.clone(), BigRational::zero()),
                        r#type,
                    )?,
                    _ => unsafe {
                        self.context
                            .f64_type()
                            .const_float_from_string(&value_str.to_string())
                    }
                    .as_any_value_enum(),
                })
            }
            Variant::Int(value) => Ok(match self.to_llvm_type(r#type)? {
                AnyTypeEnum::IntType(t) => t
                    .const_int_from_string(&value.to_string(), StringRadix::Decimal)
                    .unwrap()
                    .as_any_value_enum(),
                AnyTypeEnum::FloatType(t) => unsafe {
                    t.const_float_from_string(&value.to_string())
                        .as_any_value_enum()
                },
                //可能是复数类型
                AnyTypeEnum::StructType(_) => self.variant_to_value(
                    &Variant::Complex(
                        BigRational::from_integer(value.clone()),
                        BigRational::zero(),
                    ),
                    r#type,
                )?,
                _ => self
                    .context
                    .i32_type()
                    .const_int_from_string(&value.to_string(), StringRadix::Decimal)
                    .unwrap()
                    .as_any_value_enum(),
            }),
            Variant::Nullptr => Ok(self
                .context
                .ptr_type(AddressSpace::default())
                .const_null()
                .as_any_value_enum()),
            Variant::Array(array) => {
                let element_type = match array_element(Rc::clone(&r#type)) {
                    Some(t) => t,
                    None => {
                        return Err(Diagnostic::error()
                            .with_message(format!("'{}' is not array", r#type.borrow()))
                            .with_label(Label::primary(
                                r#type.borrow().file_id,
                                r#type.borrow().span,
                            )));
                    }
                };
                Ok(match self.to_llvm_type(&element_type)? {
                    AnyTypeEnum::ArrayType(t) => {
                        let mut values = Vec::new();
                        for value in array {
                            values.push(
                                self.variant_to_value(value, &element_type)?
                                    .into_array_value(),
                            );
                        }
                        t.const_array(&values).as_any_value_enum()
                    }
                    AnyTypeEnum::IntType(t) => {
                        let mut values = Vec::new();
                        for value in array {
                            values.push(
                                self.variant_to_value(value, &element_type)?
                                    .into_int_value(),
                            );
                        }
                        t.const_array(&values).as_any_value_enum()
                    }
                    AnyTypeEnum::FloatType(t) => {
                        let mut values = Vec::new();
                        for value in array {
                            values.push(
                                self.variant_to_value(value, &element_type)?
                                    .into_float_value(),
                            );
                        }
                        t.const_array(&values).as_any_value_enum()
                    }
                    AnyTypeEnum::PointerType(t) => {
                        let mut values = Vec::new();
                        for value in array {
                            values.push(
                                self.variant_to_value(value, &element_type)?
                                    .into_pointer_value(),
                            );
                        }
                        t.const_array(&values).as_any_value_enum()
                    }
                    AnyTypeEnum::ScalableVectorType(t) => {
                        let mut values = Vec::new();
                        for value in array {
                            values.push(
                                self.variant_to_value(value, &element_type)?
                                    .into_scalable_vector_value(),
                            );
                        }
                        t.const_array(&values).as_any_value_enum()
                    }
                    AnyTypeEnum::StructType(t) => {
                        let mut values = Vec::new();
                        for value in array {
                            values.push(
                                self.variant_to_value(value, &element_type)?
                                    .into_struct_value(),
                            );
                        }
                        t.const_array(&values).as_any_value_enum()
                    }
                    AnyTypeEnum::VectorType(t) => {
                        let mut values = Vec::new();
                        for value in array {
                            values.push(
                                self.variant_to_value(value, &element_type)?
                                    .into_vector_value(),
                            );
                        }
                        t.const_array(&values).as_any_value_enum()
                    }
                    _ => {
                        return Err(Diagnostic::error()
                            .with_message(format!(
                                "'{}' cannot be used as array element types",
                                r#type.borrow()
                            ))
                            .with_label(Label::primary(
                                r#type.borrow().file_id,
                                r#type.borrow().span,
                            )));
                    }
                })
            }
            Variant::Complex(a, b) => {
                let t = match &r#type.borrow().kind {
                    TypeKind::Complex(Some(t)) => match self.to_llvm_type(t)? {
                        AnyTypeEnum::FloatType(t) => Some(t),
                        _ => None,
                    },
                    _ => None,
                };
                if let Some(t) = t {
                    Ok(self
                        .context
                        .const_struct(
                            &[
                                unsafe { t.const_float_from_string(&to_decimal(a).to_string()) }
                                    .as_basic_value_enum(),
                                unsafe { t.const_float_from_string(&to_decimal(b).to_string()) }
                                    .as_basic_value_enum(),
                            ],
                            false,
                        )
                        .as_any_value_enum())
                } else {
                    self.variant_to_value(&Variant::Rational(a.clone()), r#type)
                }
            }
            Variant::Unknown => Err(Diagnostic::error()
                .with_message(format!("unkown value for type"))
                .with_label(Label::primary(
                    r#type.borrow().file_id,
                    r#type.borrow().span,
                ))),
        }
    }

    fn layout_to_value(
        &self,
        layout: Layout,
        path: Vec<ConstDesignation>,
        init_values: &HashMap<Vec<ConstDesignation>, Self::Value>,
    ) -> Result<Self::Value, Diagnostic<usize>> {
        if let Some(t) = init_values.get(&path) {
            Ok(*t)
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
                    bitfields.push((t.into_int_value(), child.width, child.offset));
                }
            }

            if bitfields.iter().all(|x| x.0.is_const()) {
                let mut value = 0;
                for (bitfield, width, offset) in bitfields {
                    value |= (bitfield.get_zero_extended_constant().unwrap() & ((1 << width) - 1))
                        << offset;
                }
                Ok(self
                    .to_llvm_type(&layout.r#type)?
                    .into_int_type()
                    .const_int(value, false)
                    .as_any_value_enum())
            } else {
                let mut value = self
                    .to_llvm_type(&layout.r#type)?
                    .into_int_type()
                    .const_int(0, false);
                let file_id = layout.r#type.borrow().file_id;
                let span = layout.r#type.borrow().span;
                for (bitfield, width, offset) in bitfields {
                    let mask = (1 << width) - 1;
                    let and = self.map_builder_err_with(
                        self.builder.build_and(
                            bitfield,
                            self.context.i32_type().const_int(mask, false),
                            "and",
                        ),
                        file_id,
                        span,
                    )?;
                    let shl = self.map_builder_err_with(
                        self.builder.build_left_shift(
                            and,
                            self.context.i32_type().const_int(offset as u64, false),
                            "shl",
                        ),
                        file_id,
                        span,
                    )?;
                    value = self.map_builder_err_with(
                        self.builder.build_or(value, shl, "or"),
                        file_id,
                        span,
                    )?;
                }
                Ok(value.as_any_value_enum())
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
                            fields.push(
                                BasicValueEnum::try_from(self.layout_to_value(
                                    child.clone(),
                                    path,
                                    init_values,
                                )?)
                                .map_err(|_| self.error("not a basic value"))?,
                            );
                            break 'outer;
                        }
                    }
                } else {
                    fields.push(
                        BasicValueEnum::try_from(self.layout_to_value(
                            child.clone(),
                            path,
                            init_values,
                        )?)
                        .map_err(|_| self.error("not a basic value"))?,
                    );
                }
            }

            if fields.iter().all(|x| x.is_const()) {
                Ok(self
                    .context
                    .const_struct(&fields, false)
                    .as_any_value_enum())
            } else {
                let file_id = layout.r#type.borrow().file_id;
                let span = layout.r#type.borrow().span;
                let mut value = self
                    .context
                    .struct_type(
                        &fields
                            .iter()
                            .map(|x| x.get_type())
                            .collect::<Vec<BasicTypeEnum>>(),
                        false,
                    )
                    .const_zero();
                for (i, field) in fields.iter().enumerate() {
                    value = self
                        .map_builder_err_with(
                            self.builder.build_insert_value(value, *field, i as u32, ""),
                            file_id,
                            span,
                        )?
                        .into_struct_value();
                }
                Ok(value.as_any_value_enum())
            }
        } else {
            Ok(
                match BasicTypeEnum::try_from(self.to_llvm_type(&layout.r#type)?) {
                    Ok(t) => t.const_zero().as_any_value_enum(),
                    Err(_) => {
                        return Err(Diagnostic::error()
                            .with_message(format!("not a basic type: '{}'", layout.r#type.borrow()))
                            .with_label(Label::primary(
                                layout.r#type.borrow().file_id,
                                layout.r#type.borrow().span,
                            )));
                    }
                },
            )
        }
    }

    fn decl_variable(
        &mut self,
        name: &str,
        r#type: &Rc<RefCell<Type>>,
        storage_classes: &[crate::ast::decl::StorageClass],
        init_value: Option<Self::Value>,
        vla_size: Option<Self::Value>,
    ) -> Result<Self::Value, Diagnostic<usize>> {
        let llvm_type = match BasicTypeEnum::try_from(self.to_llvm_type(r#type)?) {
            Ok(t) => t,
            Err(_) => {
                return Err(self.error(format!("{} is not a basic type", r#type.borrow())));
            }
        };

        let init_value = match init_value {
            Some(t) => Some(match BasicValueEnum::try_from(t) {
                Ok(t) => t,
                Err(_) => {
                    return Err(self.error("not a basic value"));
                }
            }),
            None => None,
        };

        let value = if self.func_values.len() == 0
            || storage_classes
                .iter()
                .any(|x| x.kind == StorageClassKind::Static || x.kind == StorageClassKind::Extern)
        {
            let value = self.module.add_global(llvm_type, None, name);

            value.set_thread_local(
                storage_classes
                    .iter()
                    .any(|x| x.kind == StorageClassKind::ThreadLocal),
            );

            if storage_classes
                .iter()
                .any(|x| x.kind == StorageClassKind::Static)
            {
                value.set_linkage(Linkage::Internal);
            } else if storage_classes
                .iter()
                .any(|x| x.kind == StorageClassKind::Extern)
            {
                value.set_linkage(Linkage::External);
            }

            if let Some(init_value) = init_value {
                if init_value.is_const() {
                    value.set_initializer(&init_value);
                } else {
                    return Err(self.error("not a constant"));
                }
            }
            value.as_any_value_enum()
        } else if r#type.borrow().is_vla() {
            let element_type = array_element(Rc::clone(&r#type)).unwrap();
            self.map_builder_err(self.builder.build_array_alloca(
                match BasicTypeEnum::try_from(self.to_llvm_type(&element_type)?) {
                    Ok(t) => t,
                    Err(_) => {
                        return Err(self.error(format!(
                            "array element not has a basic type: '{}'",
                            element_type.borrow()
                        )));
                    }
                },
                vla_size.unwrap().into_int_value(),
                name,
            ))?
            .as_any_value_enum()
        } else {
            let value = { self.map_builder_err(self.builder.build_alloca(llvm_type, name))? };
            if let Some(init_value) = init_value {
                self.map_builder_err(self.builder.build_store(value, init_value))?;
            }
            value.as_any_value_enum()
        };
        self.values.insert(
            self.lookup(name, Namespace::Ordinary).unwrap().as_ptr() as usize,
            value,
        );
        Ok(value)
    }

    fn decl_function(
        &mut self,
        name: &str,
        r#type: &Rc<RefCell<Type>>,
        storage_classes: &[crate::ast::decl::StorageClass],
        function_specs: &[crate::ast::decl::FunctionSpec],
    ) -> Result<Self::Value, Diagnostic<usize>> {
        let symbol = self.lookup(name, Namespace::Ordinary).unwrap();
        let function = if let Some(t) = self.values.get(&(symbol.as_ptr() as usize)) {
            t.into_function_value()
        } else {
            let function = self.module.add_function(
                name,
                self.to_llvm_type(r#type)?.into_function_type(),
                if function_specs
                    .iter()
                    .any(|x| x.kind == FunctionSpecKind::Inline)
                    || storage_classes
                        .iter()
                        .any(|x| x.kind == StorageClassKind::Static)
                {
                    Some(Linkage::Internal)
                } else if storage_classes
                    .iter()
                    .any(|x| x.kind == StorageClassKind::Extern)
                {
                    Some(Linkage::External)
                } else {
                    Some(Linkage::External)
                },
            );
            self.values
                .insert(symbol.as_ptr() as usize, function.as_any_value_enum());
            function
        };
        Ok(function.as_any_value_enum())
    }

    fn decl_parameter(
        &mut self,
        name: &str,
        r#type: &Rc<RefCell<Type>>,
    ) -> Result<Self::Value, Diagnostic<usize>> {
        let symbol = self.lookup(name, Namespace::Ordinary).unwrap();
        let llvm_type = match BasicTypeEnum::try_from(self.to_llvm_type(r#type)?) {
            Ok(t) => t,
            Err(_) => {
                return Err(self.error(format!("{} is not a basic type", r#type.borrow())));
            }
        };
        match &symbol.borrow().kind {
            SymbolKind::Parameter { index, .. } => {
                let param_value =
                    self.map_builder_err(self.builder.build_alloca(llvm_type, name))?;
                self.map_builder_err(
                    self.builder.build_store(
                        param_value,
                        self.func_values
                            .last()
                            .unwrap()
                            .get_nth_param(*index)
                            .unwrap(),
                    ),
                )?;
                self.values
                    .insert(symbol.as_ptr() as usize, param_value.as_any_value_enum());

                Ok(param_value.as_any_value_enum())
            }
            _ => unreachable!(),
        }
    }

    fn decl_record(
        &mut self,
        name: &str,
        r#type: &Rc<RefCell<Type>>,
    ) -> Result<(), Diagnostic<usize>> {
        let record_type = match &r#type.borrow().kind {
            TypeKind::Record {
                kind: RecordKind::Struct,
                members: Some(_),
                ..
            } => {
                let Some(layout) = compute_layout(Rc::clone(&r#type)) else {
                    return Err(
                        self.error(format!("cannot compute layout for '{}'", r#type.borrow()))
                    );
                };
                let mut field_types = Vec::new();
                for child_layout in layout.children {
                    let child_layout_ty = &child_layout.r#type;
                    let llvm_member_ty = self.to_llvm_type(child_layout_ty)?;
                    field_types.push(match llvm_member_ty {
                        AnyTypeEnum::ArrayType(_) => llvm_member_ty.into_array_type().into(),
                        AnyTypeEnum::FloatType(_) => llvm_member_ty.into_float_type().into(),
                        AnyTypeEnum::IntType(_) => llvm_member_ty.into_int_type().into(),
                        AnyTypeEnum::PointerType(_) => llvm_member_ty.into_pointer_type().into(),
                        AnyTypeEnum::ScalableVectorType(_) => {
                            llvm_member_ty.into_scalable_vector_type().into()
                        }
                        AnyTypeEnum::StructType(_) => llvm_member_ty.into_struct_type().into(),
                        AnyTypeEnum::VectorType(_) => llvm_member_ty.into_vector_type().into(),
                        _ => {
                            return Err(Diagnostic::error()
                                .with_message(format!(
                                    "invalid type for member: {}",
                                    child_layout_ty.borrow()
                                ))
                                .with_label(Label::primary(
                                    child_layout_ty.borrow().file_id,
                                    child_layout_ty.borrow().span,
                                )));
                        }
                    })
                }
                self.context
                    .struct_type(&field_types, false)
                    .as_any_type_enum()
            }
            TypeKind::Record {
                kind: RecordKind::Union,
                members: Some(_),
                ..
            } => self
                .context
                .struct_type(
                    &[self
                        .context
                        .custom_width_int_type(
                            //根据语法, union至少有一个成员, 所以union的大小肯定大于0
                            NonZero::new((r#type.borrow().size().unwrap() * 8) as u32).unwrap(),
                        )
                        .map_err(|x| self.error(x.to_string()))?
                        .as_basic_type_enum()],
                    false,
                )
                .as_any_type_enum(),
            _ => unreachable!(),
        };
        self.types.insert(
            self.lookup(name, Namespace::Tag).unwrap().as_ptr() as usize,
            record_type,
        );
        Ok(())
    }

    fn conditional_branch(
        &mut self,
        condition: &Self::Value,
        then_block: &Self::BasicBlock,
        else_block: &Self::BasicBlock,
    ) -> Result<(), Diagnostic<usize>> {
        self.map_builder_err(self.builder.build_conditional_branch(
            self.to_bool(*condition)?,
            *then_block,
            *else_block,
        ))?;
        Ok(())
    }

    fn unconditional_branch(
        &mut self,
        dest_block: &Self::BasicBlock,
    ) -> Result<(), Diagnostic<usize>> {
        self.map_builder_err(self.builder.build_unconditional_branch(*dest_block))?;
        Ok(())
    }

    fn switch(
        &mut self,
        condition: &Self::Value,
        cases: &[(Self::Value, Self::BasicBlock)],
        default: &Self::BasicBlock,
    ) -> Result<(), Diagnostic<usize>> {
        self.map_builder_err(
            self.builder.build_switch(
                condition.into_int_value(),
                *default,
                &cases
                    .iter()
                    .map(|(value, block)| (value.into_int_value(), *block))
                    .collect::<Vec<_>>(),
            ),
        )?;
        Ok(())
    }

    fn r#return(&mut self, value: Option<Self::Value>) -> Result<(), Diagnostic<usize>> {
        let ret_value: Option<&dyn BasicValue<'_>> = match &value {
            Some(t) => match t {
                AnyValueEnum::ArrayValue(t) => Some(t),
                AnyValueEnum::FloatValue(t) => Some(t),
                AnyValueEnum::IntValue(t) => Some(t),
                AnyValueEnum::PointerValue(t) => Some(t),
                AnyValueEnum::ScalableVectorValue(t) => Some(t),
                AnyValueEnum::StructValue(t) => Some(t),
                AnyValueEnum::VectorValue(t) => Some(t),
                _ => {
                    return Err(self.error("return a invalid value"));
                }
            },
            None => None,
        };
        self.map_builder_err(self.builder.build_return(ret_value))?;
        Ok(())
    }

    fn load(
        &mut self,
        ptr: &Self::Value,
        r#type: &Rc<RefCell<Type>>,
    ) -> Result<Self::Value, Diagnostic<usize>> {
        Ok(self
            .map_builder_err(self.builder.build_load(
                match BasicTypeEnum::try_from(self.to_llvm_type(r#type)?) {
                    Ok(t) => t,
                    Err(_) => {
                        return Err(
                            self.error(format!("invalid type for lvalue: {}", r#type.borrow()))
                        );
                    }
                },
                ptr.into_pointer_value(),
                "",
            ))?
            .as_any_value_enum())
    }

    fn load_var(&mut self, name: &str) -> Result<Self::Value, Diagnostic<usize>> {
        let symbol = self.lookup(name, Namespace::Ordinary).unwrap();
        Ok(*self.values.get(&(symbol.as_ptr() as usize)).unwrap())
    }

    fn load_member(
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
                let SymbolKind::Member { index, .. } =
                    &members.get(member_name).unwrap().borrow().kind
                else {
                    unreachable!();
                };
                Ok(self
                    .map_builder_err(self.builder.build_struct_gep(
                        BasicTypeEnum::try_from(self.to_llvm_type(record_type)?).unwrap(),
                        target_value.into_pointer_value(),
                        *index as u32,
                        "",
                    ))?
                    .as_any_value_enum())
            }
            TypeKind::Record {
                kind: RecordKind::Union,
                ..
            } => Ok(target_value.as_any_value_enum()),
            _ => unreachable!(),
        }
    }

    fn load_compound_literal(
        &mut self,
        r#type: &Rc<RefCell<Type>>,
        storage_classes: &[crate::ast::decl::StorageClass],
        init_value: &Self::Value,
    ) -> Result<Self::Value, Diagnostic<usize>> {
        let init_value = match BasicValueEnum::try_from(*init_value) {
            Ok(t) => t,
            Err(_) => {
                return Err(self.error("not a basic value"));
            }
        };

        let ty = match BasicTypeEnum::try_from(self.to_llvm_type(r#type)?) {
            Ok(t) => t,
            Err(_) => {
                return Err(Diagnostic::error()
                    .with_message("not a basic type")
                    .with_label(Label::primary(
                        r#type.borrow().file_id,
                        r#type.borrow().span,
                    )));
            }
        };

        if self.func_values.len() == 0
            || storage_classes
                .iter()
                .any(|x| x.kind == StorageClassKind::Static)
        {
            let value = self.module.add_global(ty, None, "compoundliteral");

            value.set_thread_local(
                storage_classes
                    .iter()
                    .any(|x| x.kind == StorageClassKind::ThreadLocal),
            );

            if storage_classes
                .iter()
                .any(|x| x.kind == StorageClassKind::Static)
            {
                value.set_linkage(Linkage::Internal);
            }

            if init_value.is_const() {
                value.set_initializer(&init_value);
            } else {
                return Err(self.error("not a constant"));
            }
            Ok(value.as_pointer_value().as_any_value_enum())
        } else {
            let value = self.map_builder_err(self.builder.build_alloca(ty, "compoundliteral"))?;
            self.map_builder_err(self.builder.build_store(value, init_value))?;
            Ok(value.as_any_value_enum())
        }
    }

    fn load_bitfield(
        &mut self,
        value: &Self::Value,
        width: usize,
        offset: usize,
    ) -> Result<Self::Value, Diagnostic<usize>> {
        let mask = self.context.i32_type().const_int((1 << width) - 1, false);
        let lshr = self.map_builder_err(self.builder.build_right_shift(
            value.into_int_value(),
            self.context.i32_type().const_int(offset as u64, false),
            false,
            "lshr",
        ))?;
        Ok(self
            .map_builder_err(self.builder.build_and(lshr, mask, ""))?
            .as_any_value_enum())
    }

    fn store(&mut self, ptr: &Self::Value, value: &Self::Value) -> Result<(), Diagnostic<usize>> {
        self.map_builder_err(self.builder.build_store(
            ptr.into_pointer_value(),
            BasicValueEnum::try_from(*value).unwrap(),
        ))?;
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
        let old_value = self.map_builder_err(self.builder.build_load(
            BasicTypeEnum::try_from(self.to_llvm_type(r#type)?).unwrap(),
            ptr.into_pointer_value(),
            "load",
        ))?;
        let masked_value = self.map_builder_err(
            self.builder.build_and(
                old_value.into_int_value(),
                self.context
                    .i32_type()
                    .const_int(!(((1 << width) - 1) << offset), false),
                "and_mask",
            ),
        )?;
        //清除value的高位
        let mut value = self
            .map_builder_err(self.builder.build_and(
                value.into_int_value(),
                self.context.i32_type().const_int((1 << width) - 1, false),
                "",
            ))?
            .as_any_value_enum();
        let shl = self.map_builder_err(self.builder.build_left_shift(
            value.into_int_value(),
            self.context.i32_type().const_int(offset as u64, false),
            "shl",
        ))?;
        value = self
            .map_builder_err(self.builder.build_or(shl, masked_value, "or"))?
            .as_any_value_enum();

        self.store(ptr, &value)?;
        Ok(())
    }

    fn cast(
        &mut self,
        target_value: &Self::Value,
        r#type: &Rc<RefCell<Type>>,
        method: CastMethod,
    ) -> Result<Self::Value, Diagnostic<usize>> {
        match method {
            CastMethod::Nothing | CastMethod::PtrToPtr => Ok(*target_value),
            CastMethod::FuncToPtr => Ok(match target_value {
                AnyValueEnum::FunctionValue(t) => {
                    //实际上在调用as_any_value_enum后值又变成了FunctionValue
                    t.as_global_value().as_pointer_value().as_any_value_enum()
                }
                _ => unreachable!(),
            }),
            CastMethod::ArrayToPtr => {
                Ok(self
                    .map_builder_err(unsafe {
                        self.builder.build_in_bounds_gep(
                            //如果是数组, 那结果是array type, 如果是vla, 那结果是pointer type
                            BasicTypeEnum::try_from(self.to_llvm_type(r#type)?).unwrap(),
                            //target 一般是左值, 而左值一定是PointerValue
                            target_value.into_pointer_value(),
                            &[self.context.i64_type().const_int(0, false)],
                            "",
                        )
                    })?
                    .as_any_value_enum())
            }
            CastMethod::LToRValue => unreachable!(),
            CastMethod::ComplexExtend | CastMethod::ComplexTrunc => {
                let complex_type = self.to_llvm_type(r#type)?.into_struct_type();
                let to_type = complex_type
                    .get_field_type_at_index(0)
                    .unwrap()
                    .into_float_type();
                let (real, imag) = self.map_builder_err(self.extract_real_imag(*target_value))?;
                let op = match method {
                    CastMethod::ComplexExtend => InstructionOpcode::FPExt,
                    CastMethod::ComplexTrunc => InstructionOpcode::FPTrunc,
                    _ => unreachable!(),
                };
                let cast_real =
                    self.map_builder_err(self.builder.build_cast(op, real, to_type, "cast_real"))?;
                let cast_imag =
                    self.map_builder_err(self.builder.build_cast(op, imag, to_type, "cast_real"))?;
                Ok(self
                    .map_builder_err(self.builder.build_insert_value(
                        self.map_builder_err(self.builder.build_insert_value(
                            complex_type.const_named_struct(&[
                                to_type.const_zero().as_basic_value_enum(),
                                to_type.const_zero().as_basic_value_enum(),
                            ]),
                            cast_real,
                            0,
                            "",
                        ))?,
                        cast_imag,
                        1,
                        "",
                    ))?
                    .as_any_value_enum())
            }
            CastMethod::ToBool => Ok(self.to_bool(*target_value)?.as_any_value_enum()),
            CastMethod::FloatToComplex => {
                let complex_type = self.to_llvm_type(r#type)?.into_struct_type();
                Ok(self
                    .map_builder_err(self.builder.build_insert_value(
                        complex_type.const_zero(),
                        target_value.into_float_value(),
                        0,
                        "",
                    ))?
                    .as_any_value_enum())
            }
            CastMethod::ComplexToFloat => {
                let (real, _) = self.map_builder_err(self.extract_real_imag(*target_value))?;
                Ok(real.as_any_value_enum())
            }
            _ => Ok(self
                .map_builder_err(self.builder.build_cast(
                    match method {
                        CastMethod::PtrToInt => InstructionOpcode::PtrToInt,
                        CastMethod::IntToPtr => InstructionOpcode::IntToPtr,
                        CastMethod::FloatTrunc => InstructionOpcode::FPTrunc,
                        CastMethod::FloatExtend => InstructionOpcode::FPExt,
                        CastMethod::FloatToSInt => InstructionOpcode::FPToSI,
                        CastMethod::FloatToUInt => InstructionOpcode::FPToUI,
                        CastMethod::SIntToFloat => InstructionOpcode::SIToFP,
                        CastMethod::UIntToFloat => InstructionOpcode::UIToFP,
                        CastMethod::SignedExtend => InstructionOpcode::SExt,
                        CastMethod::ZeroExtand => InstructionOpcode::ZExt,
                        CastMethod::IntTrunc => InstructionOpcode::Trunc,
                        _ => unreachable!(),
                    },
                    match BasicValueEnum::try_from(*target_value) {
                        Ok(t) => t,
                        Err(_) => {
                            return Err(
                                self.error(format!("cast from a invalid value: {target_value}"))
                            );
                        }
                    },
                    match BasicTypeEnum::try_from(self.to_llvm_type(r#type)?) {
                        Ok(t) => t,
                        Err(_) => {
                            return Err(
                                self.error(format!("cast to a invalid type: {}", r#type.borrow()))
                            );
                        }
                    },
                    "",
                ))?
                .as_any_value_enum()),
        }
    }

    fn subscript(
        &mut self,
        target: &Self::Value,
        index: &Self::Value,
        r#type: &Rc<RefCell<Type>>,
    ) -> Result<Self::Value, Diagnostic<usize>> {
        match (target, index) {
            (AnyValueEnum::PointerValue(target), AnyValueEnum::IntValue(index))
            | (AnyValueEnum::IntValue(index), AnyValueEnum::PointerValue(target)) => Ok(self
                .map_builder_err(unsafe {
                    self.builder.build_in_bounds_gep(
                        BasicTypeEnum::try_from(self.to_llvm_type(r#type)?).unwrap(),
                        *target,
                        &[*index],
                        "",
                    )
                })?
                .as_any_value_enum()),
            _ => unreachable!(),
        }
    }

    fn call(
        &mut self,
        target: &Self::Value,
        args: &[Self::Value],
        r#type: &Rc<RefCell<Type>>,
    ) -> Result<Self::Value, Diagnostic<usize>> {
        let args = args
            .iter()
            .map(|x| BasicValueEnum::try_from(*x).unwrap().into())
            .collect::<Vec<_>>();
        match target {
            AnyValueEnum::PointerValue(t) => Ok(self
                .map_builder_err(
                    self.builder.build_indirect_call(
                        self.to_llvm_type(&pointee(Rc::clone(r#type)).unwrap())?
                            .into_function_type(),
                        *t,
                        &args,
                        "",
                    ),
                )?
                .as_any_value_enum()),
            AnyValueEnum::FunctionValue(t) => Ok(self
                .map_builder_err(self.builder.build_direct_call(*t, &args, ""))?
                .as_any_value_enum()),
            _ => unreachable!(),
        }
    }

    fn phi(
        &mut self,
        r#type: &Rc<RefCell<Type>>,
        incomings: &[(Self::Value, Self::BasicBlock)],
    ) -> Result<Self::Value, Diagnostic<usize>> {
        let phi = self.map_builder_err(self.builder.build_phi(
            BasicTypeEnum::try_from(self.to_llvm_type(r#type)?).unwrap(),
            "",
        ))?;
        let mut incoming = vec![];
        for (value, block) in incomings {
            incoming.push((BasicValueEnum::try_from(*value).unwrap(), *block));
        }
        let mut final_incoming: Vec<(&dyn BasicValue<'_>, BasicBlock<'_>)> = vec![];
        for (value, block) in &incoming {
            final_incoming.push((value, *block));
        }
        phi.add_incoming(&final_incoming);

        Ok(phi.as_any_value_enum())
    }

    fn unaryop(
        &mut self,
        op: UnaryOpKind,
        operand_value: &Self::Value,
        operand_type: &Rc<RefCell<Type>>,
    ) -> Result<Self::Value, Diagnostic<usize>> {
        match op {
            UnaryOpKind::AddressOf => match operand_value {
                AnyValueEnum::FunctionValue(t) => {
                    Ok(t.as_global_value().as_pointer_value().as_any_value_enum())
                }
                //对于左值, operand_value一定是指针
                _ => Ok(*operand_value),
            },
            UnaryOpKind::Dereference => Ok(self
                .map_builder_err(self.builder.build_load(
                    self.to_llvm_type(operand_type)?.into_pointer_type(),
                    operand_value.into_pointer_value(),
                    "",
                ))?
                .as_any_value_enum()),
            UnaryOpKind::Not => Ok(self
                .map_builder_err(self.builder.build_int_compare(
                    IntPredicate::EQ,
                    operand_value.into_int_value(),
                    self.context.bool_type().const_int(0, false),
                    "",
                ))?
                .as_any_value_enum()),
            UnaryOpKind::Positive => Ok(*operand_value),
            UnaryOpKind::Negative => {
                match &operand_type.borrow().kind {
                    t if t.is_integer() => Ok(self
                        .map_builder_err(
                            self.builder
                                .build_int_neg(operand_value.into_int_value(), ""),
                        )?
                        .as_any_value_enum()),
                    t if t.is_real_float() => Ok(self
                        .map_builder_err(
                            self.builder
                                .build_float_neg(operand_value.into_float_value(), ""),
                        )?
                        .as_any_value_enum()),
                    t if t.is_complex() => {
                        let operand_value = operand_value.into_struct_value();

                        let real = self
                            .map_builder_err(self.builder.build_extract_value(
                                operand_value,
                                0,
                                "real",
                            ))?
                            .into_float_value();
                        let imag = self
                            .map_builder_err(self.builder.build_extract_value(
                                operand_value,
                                1,
                                "imag",
                            ))?
                            .into_float_value();

                        let neg_real =
                            self.map_builder_err(self.builder.build_float_neg(real, "neg_real"))?;
                        let neg_imag =
                            self.map_builder_err(self.builder.build_float_neg(imag, "neg_image"))?;

                        Ok(self
                            .map_builder_err(self.builder.build_insert_value(
                                self.map_builder_err(self.builder.build_insert_value(
                                    operand_value,
                                    neg_imag,
                                    1,
                                    "",
                                ))?,
                                neg_real,
                                0,
                                "",
                            ))?
                            .as_any_value_enum())
                    }
                    //TODO 十进制浮点数
                    _ => todo!(),
                }
            }
            UnaryOpKind::BitNot => Ok(self
                .map_builder_err(self.builder.build_not(operand_value.into_int_value(), ""))?
                .as_any_value_enum()),
            UnaryOpKind::PostfixDec
            | UnaryOpKind::PrefixDec
            | UnaryOpKind::PostfixInc
            | UnaryOpKind::PrefixInc => {
                let offset: i64 = match op {
                    UnaryOpKind::PostfixDec | UnaryOpKind::PrefixDec => -1,
                    UnaryOpKind::PrefixInc | UnaryOpKind::PostfixInc => 1,
                    _ => unreachable!(),
                };
                //操作数是左值且没有进行过左值转换
                let old_value = self.map_builder_err(self.builder.build_load(
                    BasicTypeEnum::try_from(self.to_llvm_type(operand_type)?).unwrap(),
                    operand_value.into_pointer_value(),
                    "",
                ))?;
                let new_value = match &operand_type.borrow().kind {
                    t if t.is_integer() => self
                        .map_builder_err(
                            self.builder.build_int_add(
                                old_value.into_int_value(),
                                old_value
                                    .get_type()
                                    .into_int_type()
                                    .const_int(offset as u64, true),
                                "",
                            ),
                        )?
                        .as_any_value_enum(),
                    t if t.is_real_float() => self
                        .map_builder_err(
                            self.builder.build_float_add(
                                old_value.into_float_value(),
                                old_value
                                    .get_type()
                                    .into_float_type()
                                    .const_float(offset as f64),
                                "",
                            ),
                        )?
                        .as_any_value_enum(),
                    t if t.is_pointer() => self
                        .map_builder_err(unsafe {
                            self.builder.build_in_bounds_gep(
                                match BasicTypeEnum::try_from(
                                    self.to_llvm_type(&pointee(Rc::clone(operand_type)).unwrap())?,
                                ) {
                                    Ok(t) => t,
                                    Err(_) => {
                                        return Err(Diagnostic::error()
                                            .with_message(format!("not point to a basic type"))
                                            .with_label(Label::primary(
                                                operand_type.borrow().file_id,
                                                operand_type.borrow().span,
                                            )));
                                    }
                                },
                                old_value.into_pointer_value(),
                                &[self.context.i64_type().const_int(offset as u64, true)],
                                "",
                            )
                        })?
                        .as_any_value_enum(),
                    _ => unreachable!(),
                };
                self.map_builder_err(self.builder.build_store(
                    //操作数本身就是左值
                    operand_value.into_pointer_value(),
                    BasicValueEnum::try_from(new_value).unwrap(),
                ))?;
                match op {
                    UnaryOpKind::PostfixInc | UnaryOpKind::PostfixDec => {
                        Ok(old_value.as_any_value_enum())
                    }
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
                    (a, b) if a.is_integer() && b.is_integer() => Ok(self
                        .map_builder_err(self.builder.build_int_compare(
                            match op {
                                //进行常用算术转换后a和b的符号应该一样
                                BinOpKind::Ge => {
                                    if a.is_unsigned().unwrap() {
                                        IntPredicate::UGE
                                    } else {
                                        IntPredicate::SGE
                                    }
                                }
                                BinOpKind::Gt => {
                                    if a.is_unsigned().unwrap() {
                                        IntPredicate::UGT
                                    } else {
                                        IntPredicate::SGT
                                    }
                                }
                                BinOpKind::Le => {
                                    if a.is_unsigned().unwrap() {
                                        IntPredicate::ULE
                                    } else {
                                        IntPredicate::SLE
                                    }
                                }
                                BinOpKind::Lt => {
                                    if a.is_unsigned().unwrap() {
                                        IntPredicate::ULT
                                    } else {
                                        IntPredicate::SLT
                                    }
                                }
                                _ => unreachable!(),
                            },
                            left_value.into_int_value(),
                            right_value.into_int_value(),
                            "",
                        ))?
                        .as_any_value_enum()),
                    (a, b) if a.is_real_float() && b.is_real_float() => Ok(self
                        .map_builder_err(self.builder.build_float_compare(
                            match op {
                                BinOpKind::Ge => FloatPredicate::OGE,
                                BinOpKind::Gt => FloatPredicate::OGT,
                                BinOpKind::Le => FloatPredicate::OLE,
                                BinOpKind::Lt => FloatPredicate::OLT,
                                _ => unreachable!(),
                            },
                            left_value.into_float_value(),
                            right_value.into_float_value(),
                            "",
                        ))?
                        .as_any_value_enum()),
                    (a, b) if a.is_pointer() && b.is_pointer() => Ok(self
                        .map_builder_err(self.builder.build_int_compare(
                            match op {
                                BinOpKind::Ge => IntPredicate::UGE,
                                BinOpKind::Gt => IntPredicate::UGT,
                                BinOpKind::Le => IntPredicate::ULE,
                                BinOpKind::Lt => IntPredicate::ULT,
                                _ => unreachable!(),
                            },
                            left_value.into_pointer_value(),
                            right_value.into_pointer_value(),
                            "",
                        ))?
                        .as_any_value_enum()),
                    _ => unreachable!(),
                }
            }
            BinOpKind::Eq | BinOpKind::Neq => {
                match (&left_type.borrow().kind, &right_type.borrow().kind) {
                    (a, b) if a.is_integer() && b.is_integer() => Ok(self
                        .map_builder_err(self.builder.build_int_compare(
                            match op {
                                BinOpKind::Eq => IntPredicate::EQ,
                                BinOpKind::Neq => IntPredicate::NE,
                                _ => unreachable!(),
                            },
                            left_value.into_int_value(),
                            right_value.into_int_value(),
                            "",
                        ))?
                        .as_any_value_enum()),
                    (a, b) if a.is_real_float() && b.is_real_float() => Ok(self
                        .map_builder_err(self.builder.build_float_compare(
                            match op {
                                BinOpKind::Eq => FloatPredicate::OEQ,
                                BinOpKind::Neq => FloatPredicate::ONE,
                                _ => unreachable!(),
                            },
                            left_value.into_float_value(),
                            right_value.into_float_value(),
                            "",
                        ))?
                        .as_any_value_enum()),
                    (a, b)
                        if (a.is_pointer() || a.is_nullptr())
                            && (b.is_pointer() || b.is_nullptr()) =>
                    {
                        Ok(self
                            .map_builder_err(self.builder.build_int_compare(
                                match op {
                                    BinOpKind::Eq => IntPredicate::EQ,
                                    BinOpKind::Neq => IntPredicate::NE,
                                    _ => unreachable!(),
                                },
                                left_value.into_pointer_value(),
                                right_value.into_pointer_value(),
                                "",
                            ))?
                            .as_any_value_enum())
                    }
                    (a, b)
                        if (a.is_complex() && b.is_real_float())
                            || (b.is_complex() && a.is_real_float())
                            || (a.is_complex() && b.is_complex()) =>
                    {
                        let (real1, imag1) =
                            self.map_builder_err(self.extract_real_imag(*left_value))?;
                        let (real2, imag2) =
                            self.map_builder_err(self.extract_real_imag(*right_value))?;

                        let op = match op {
                            BinOpKind::Eq => FloatPredicate::OEQ,
                            BinOpKind::Neq => FloatPredicate::ONE,
                            _ => unreachable!(),
                        };

                        let cmp_real = self.map_builder_err(
                            self.builder
                                .build_float_compare(op, real1, real2, "cmp_real"),
                        )?;
                        let cmp_imag = self.map_builder_err(
                            self.builder
                                .build_float_compare(op, imag1, imag2, "cmp_imag"),
                        )?;
                        Ok(self
                            .map_builder_err(self.builder.build_and(cmp_real, cmp_imag, ""))?
                            .as_any_value_enum())
                    }
                    //TODO 十进制浮点数
                    _ => todo!(),
                }
            }
            BinOpKind::Add => {
                match (&left_type.borrow().kind, &right_type.borrow().kind) {
                    (a, b) if a.is_integer() && b.is_integer() => Ok(self
                        .map_builder_err(self.builder.build_int_add(
                            left_value.into_int_value(),
                            right_value.into_int_value(),
                            "",
                        ))?
                        .as_any_value_enum()),
                    (a, b) if a.is_real_float() && b.is_real_float() => Ok(self
                        .map_builder_err(self.builder.build_float_add(
                            left_value.into_float_value(),
                            right_value.into_float_value(),
                            "",
                        ))?
                        .as_any_value_enum()),
                    (TypeKind::Pointer(pointee), b) if b.is_integer() => Ok(self
                        .map_builder_err(unsafe {
                            self.builder.build_in_bounds_gep(
                                BasicTypeEnum::try_from(self.to_llvm_type(pointee)?).unwrap(),
                                left_value.into_pointer_value(),
                                &[right_value.into_int_value()],
                                "",
                            )
                        })?
                        .as_any_value_enum()),
                    (a, TypeKind::Pointer(pointee)) if a.is_integer() => Ok(self
                        .map_builder_err(unsafe {
                            self.builder.build_in_bounds_gep(
                                BasicTypeEnum::try_from(self.to_llvm_type(pointee)?).unwrap(),
                                right_value.into_pointer_value(),
                                &[left_value.into_int_value()],
                                "",
                            )
                        })?
                        .as_any_value_enum()),
                    (a, b)
                        if (a.is_complex() && b.is_real_float())
                            || (b.is_complex() && a.is_real_float())
                            || (a.is_complex() && b.is_complex()) =>
                    {
                        let complex_type = if a.is_complex() {
                            left_value.into_struct_value()
                        } else if b.is_complex() {
                            right_value.into_struct_value()
                        } else {
                            unreachable!()
                        };

                        let (real1, imag1) =
                            self.map_builder_err(self.extract_real_imag(*left_value))?;
                        let (real2, imag2) =
                            self.map_builder_err(self.extract_real_imag(*right_value))?;

                        let add_real = self.map_builder_err(
                            self.builder.build_float_add(real1, real2, "add_real"),
                        )?;
                        let add_imag = self.map_builder_err(
                            self.builder.build_float_add(imag1, imag2, "add_imag"),
                        )?;

                        Ok(self
                            .map_builder_err((|| {
                                self.builder.build_insert_value(
                                    self.builder
                                        .build_insert_value(complex_type, add_real, 0, "")?
                                        .into_struct_value(),
                                    add_imag,
                                    1,
                                    "",
                                )
                            })())?
                            .as_any_value_enum())
                    }
                    //TODO 十进制浮点数
                    _ => todo!(),
                }
            }
            BinOpKind::Sub => {
                match (&left_type.borrow().kind, &right_type.borrow().kind) {
                    (a, b) if a.is_integer() && b.is_integer() => Ok(self
                        .map_builder_err(self.builder.build_int_sub(
                            left_value.into_int_value(),
                            right_value.into_int_value(),
                            "",
                        ))?
                        .as_any_value_enum()),
                    (a, b) if a.is_real_float() && b.is_real_float() => Ok(self
                        .map_builder_err(self.builder.build_float_sub(
                            left_value.into_float_value(),
                            right_value.into_float_value(),
                            "",
                        ))?
                        .as_any_value_enum()),
                    (TypeKind::Pointer(pointee), b) if b.is_integer() => {
                        let right_neg = self.map_builder_err(
                            self.builder.build_int_neg(right_value.into_int_value(), ""),
                        )?;
                        Ok(self
                            .map_builder_err(unsafe {
                                self.builder.build_in_bounds_gep(
                                    BasicTypeEnum::try_from(self.to_llvm_type(pointee)?).unwrap(),
                                    left_value.into_pointer_value(),
                                    &[right_neg],
                                    "",
                                )
                            })?
                            .as_any_value_enum())
                    }
                    (a, b)
                        if (a.is_complex() && b.is_real_float())
                            || (b.is_complex() && a.is_real_float())
                            || (a.is_complex() && b.is_complex()) =>
                    {
                        let complex_type = if a.is_complex() {
                            left_value.into_struct_value()
                        } else if b.is_complex() {
                            right_value.into_struct_value()
                        } else {
                            unreachable!()
                        };

                        let (real1, imag1) =
                            self.map_builder_err(self.extract_real_imag(*left_value))?;
                        let (real2, imag2) =
                            self.map_builder_err(self.extract_real_imag(*right_value))?;

                        let sub_real = self.map_builder_err(
                            self.builder.build_float_sub(real1, real2, "sub_real"),
                        )?;
                        let sub_imag = self.map_builder_err(
                            self.builder.build_float_sub(imag1, imag2, "sub_imag"),
                        )?;

                        Ok(self
                            .map_builder_err((|| {
                                self.builder.build_insert_value(
                                    self.builder
                                        .build_insert_value(complex_type, sub_real, 0, "")?
                                        .into_struct_value(),
                                    sub_imag,
                                    1,
                                    "",
                                )
                            })())?
                            .as_any_value_enum())
                    }
                    //TODO 十进制浮点数
                    _ => todo!(),
                }
            }
            BinOpKind::Mul | BinOpKind::Div | BinOpKind::Mod => {
                match (&left_type.borrow().kind, &right_type.borrow().kind) {
                    (a, b) if a.is_integer() && b.is_integer() => Ok(self
                        .map_builder_err(match op {
                            BinOpKind::Mul => self.builder.build_int_mul(
                                left_value.into_int_value(),
                                right_value.into_int_value(),
                                "",
                            ),
                            //经过常用算术转换后a和b的符号应该相同
                            BinOpKind::Div => {
                                if a.is_unsigned().unwrap() {
                                    self.builder.build_int_unsigned_div(
                                        left_value.into_int_value(),
                                        right_value.into_int_value(),
                                        "",
                                    )
                                } else {
                                    self.builder.build_int_signed_div(
                                        left_value.into_int_value(),
                                        right_value.into_int_value(),
                                        "",
                                    )
                                }
                            }
                            BinOpKind::Mod => {
                                if a.is_unsigned().unwrap() {
                                    self.builder.build_int_unsigned_rem(
                                        left_value.into_int_value(),
                                        right_value.into_int_value(),
                                        "",
                                    )
                                } else {
                                    self.builder.build_int_signed_rem(
                                        left_value.into_int_value(),
                                        right_value.into_int_value(),
                                        "",
                                    )
                                }
                            }
                            _ => unreachable!(),
                        })?
                        .as_any_value_enum()),
                    (a, b) if a.is_real_float() && b.is_real_float() => Ok(self
                        .map_builder_err(match op {
                            BinOpKind::Mul => self.builder.build_float_mul(
                                left_value.into_float_value(),
                                right_value.into_float_value(),
                                "",
                            ),
                            BinOpKind::Div => self.builder.build_float_div(
                                left_value.into_float_value(),
                                right_value.into_float_value(),
                                "",
                            ),
                            _ => unreachable!(),
                        })?
                        .as_any_value_enum()),
                    (a, b)
                        if (a.is_complex() && b.is_real_float())
                            || (b.is_complex() && a.is_real_float())
                            || (a.is_complex() && b.is_complex()) =>
                    {
                        let complex_type = if a.is_complex() {
                            left_value.into_struct_value()
                        } else if b.is_complex() {
                            right_value.into_struct_value()
                        } else {
                            unreachable!()
                        };

                        let (real1, imag1) =
                            self.map_builder_err(self.extract_real_imag(*left_value))?;
                        let (real2, imag2) =
                            self.map_builder_err(self.extract_real_imag(*right_value))?;

                        let new_real = match op {
                            BinOpKind::Mul => self.map_builder_err((|| {
                                self.builder.build_float_sub(
                                    self.builder.build_float_mul(real1, real2, "")?,
                                    self.builder.build_float_mul(imag1, imag2, "")?,
                                    "new_real",
                                )
                            })(
                            ))?,
                            BinOpKind::Div => self.map_builder_err((|| {
                                self.builder.build_float_div(
                                    self.builder.build_float_add(
                                        self.builder.build_float_mul(real1, real2, "")?,
                                        self.builder.build_float_mul(imag1, imag2, "")?,
                                        "",
                                    )?,
                                    self.builder.build_float_add(
                                        self.builder.build_float_mul(real2, real2, "")?,
                                        self.builder.build_float_mul(imag2, imag2, "")?,
                                        "",
                                    )?,
                                    "new_real",
                                )
                            })(
                            ))?,
                            _ => unreachable!(),
                        };
                        let new_imag = match op {
                            BinOpKind::Mul => self.map_builder_err((|| {
                                self.builder.build_float_add(
                                    self.builder.build_float_mul(real1, imag2, "")?,
                                    self.builder.build_float_mul(real2, imag1, "")?,
                                    "new_imag",
                                )
                            })(
                            ))?,
                            BinOpKind::Div => self.map_builder_err((|| {
                                self.builder.build_float_div(
                                    self.builder.build_float_sub(
                                        self.builder.build_float_mul(imag1, real2, "")?,
                                        self.builder.build_float_mul(real1, imag2, "")?,
                                        "",
                                    )?,
                                    self.builder.build_float_add(
                                        self.builder.build_float_mul(real2, real2, "")?,
                                        self.builder.build_float_mul(imag2, imag2, "")?,
                                        "",
                                    )?,
                                    "new_imag",
                                )
                            })(
                            ))?,
                            _ => unreachable!(),
                        };

                        Ok(self
                            .map_builder_err((|| {
                                self.builder.build_insert_value(
                                    self.builder
                                        .build_insert_value(complex_type, new_real, 0, "")?
                                        .into_struct_value(),
                                    new_imag,
                                    1,
                                    "",
                                )
                            })())?
                            .as_any_value_enum())
                    }
                    //TODO 十进制浮点数
                    _ => todo!(),
                }
            }
            BinOpKind::BitAnd
            | BinOpKind::BitOr
            | BinOpKind::BitXOr
            | BinOpKind::LShift
            | BinOpKind::RShift => match (&left_type.borrow().kind, &right_type.borrow().kind) {
                (a, b) if a.is_integer() && b.is_integer() => Ok(self
                    .map_builder_err(match op {
                        BinOpKind::BitAnd => self.builder.build_and(
                            left_value.into_int_value(),
                            right_value.into_int_value(),
                            "",
                        ),
                        BinOpKind::BitOr => self.builder.build_or(
                            left_value.into_int_value(),
                            right_value.into_int_value(),
                            "",
                        ),
                        BinOpKind::BitXOr => self.builder.build_xor(
                            left_value.into_int_value(),
                            right_value.into_int_value(),
                            "",
                        ),
                        BinOpKind::LShift => self.builder.build_left_shift(
                            left_value.into_int_value(),
                            right_value.into_int_value(),
                            "",
                        ),
                        BinOpKind::RShift => self.builder.build_right_shift(
                            left_value.into_int_value(),
                            right_value.into_int_value(),
                            !a.is_unsigned().unwrap(),
                            "",
                        ),
                        _ => unreachable!(),
                    })?
                    .as_any_value_enum()),
                _ => unreachable!(),
            },

            BinOpKind::Comma => Ok(right_value.as_any_value_enum()),
            _ => unreachable!(),
        }
    }
}
