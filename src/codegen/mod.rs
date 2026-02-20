pub mod gen_decl;
pub mod gen_expr;
pub mod gen_init;
pub mod gen_stmt;
#[cfg(test)]
pub mod tests;

use crate::{
    ast::TranslationUnit,
    ctype::{Type, TypeKind, array_element},
    symtab::{Namespace, Symbol, SymbolTable},
    variant::{Variant, to_decimal},
};
use codespan_reporting::diagnostic::{Diagnostic, Label};
use inkwell::{
    AddressSpace,
    basic_block::BasicBlock,
    builder::Builder,
    context::Context,
    module::Module,
    types::{AnyType, AnyTypeEnum, BasicType, StringRadix},
    values::{AnyValue, AnyValueEnum, BasicValue, FunctionValue, IntValue},
};
use num::ToPrimitive;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub struct CodeGen<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub symtab: Vec<Rc<RefCell<SymbolTable>>>,
    //break跳出的代码块
    pub break_blocks: Vec<BasicBlock<'ctx>>,
    //continue跳入的代码块
    pub continue_blocks: Vec<BasicBlock<'ctx>>,
    pub func_values: Vec<FunctionValue<'ctx>>,
    //label symbol到对应block的映射
    pub label_blocks: HashMap<usize, BasicBlock<'ctx>>,
    pub symbol_values: HashMap<usize, AnyValueEnum<'ctx>>,
    pub cases_or_default: Vec<Vec<(Option<IntValue<'ctx>>, BasicBlock<'ctx>)>>,
    pub record_types: HashMap<usize, AnyTypeEnum<'ctx>>,
}

impl<'ctx> CodeGen<'ctx> {
    pub fn new(context: &'ctx Context, module: Module<'ctx>) -> CodeGen<'ctx> {
        CodeGen {
            context,
            module,
            builder: context.create_builder(),
            symtab: vec![],
            break_blocks: vec![],
            continue_blocks: vec![],
            func_values: vec![],
            label_blocks: HashMap::new(),
            symbol_values: HashMap::new(),
            cases_or_default: vec![],
            record_types: HashMap::new(),
        }
    }

    pub fn r#gen(&mut self, ast: Rc<RefCell<TranslationUnit>>) -> Result<(), Diagnostic<usize>> {
        if let Some(symtab) = &ast.borrow().symtab {
            self.enter_scope(Rc::clone(symtab));
        }
        for decl in &ast.borrow().decls {
            self.visit_declaration(Rc::clone(decl))?;
        }
        if let Some(_) = ast.borrow().symtab {
            self.leave_scope();
        }
        Ok(())
    }

    pub fn enter_scope(&mut self, symtab: Rc<RefCell<SymbolTable>>) {
        self.symtab.push(symtab);
    }

    pub fn leave_scope(&mut self) {
        self.symtab.pop();
    }

    pub fn lookup(&self, namespace: Namespace, name: &String) -> Option<Rc<RefCell<Symbol>>> {
        self.symtab.last().unwrap().borrow().lookup(namespace, name)
    }

    pub fn to_llvm_type(
        &self,
        r#type: Rc<RefCell<Type>>,
    ) -> Result<AnyTypeEnum<'ctx>, Diagnostic<usize>> {
        Ok(match &r#type.borrow().kind {
            TypeKind::Void => self.context.void_type().as_any_type_enum(),
            TypeKind::Bool => self.context.bool_type().as_any_type_enum(),
            TypeKind::Float => self.context.f32_type().as_any_type_enum(),
            TypeKind::Double => self.context.f64_type().as_any_type_enum(),
            TypeKind::LongDouble => self.context.f128_type().as_any_type_enum(),
            TypeKind::Complex(r#type) => {
                let t = if let Some(t) = r#type {
                    match self.to_llvm_type(Rc::clone(t))? {
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
            | TypeKind::Auto(Some(r#type)) => self.to_llvm_type(Rc::clone(r#type))?,
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
            } => self.to_llvm_type(Rc::clone(&expr.borrow().r#type))?,
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
                    Variant::Unknown => {
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
                let llvm_elem_ty = self.to_llvm_type(Rc::clone(element_type))?;
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
                    let llvm_param_ty = self.to_llvm_type(Rc::clone(parameter_type))?;

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
                let llvm_ret_ty = self.to_llvm_type(Rc::clone(return_type))?;
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
                .record_types
                .get(&(self.lookup(Namespace::Tag, name).unwrap().as_ptr() as usize))
                .unwrap(),
            t if t.is_integer() => match r#type.borrow().size() {
                Some(1) => self.context.i8_type().as_any_type_enum(),
                Some(2) => self.context.i16_type().as_any_type_enum(),
                Some(4) => self.context.i32_type().as_any_type_enum(),
                Some(8) => self.context.i64_type().as_any_type_enum(),
                Some(16) => self.context.i128_type().as_any_type_enum(),
                Some(t) => self
                    .context
                    .custom_width_int_type((t * 8) as u32)
                    .as_any_type_enum(),
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

    pub fn to_llvm_value(
        &self,
        value: Variant,
        r#type: Rc<RefCell<Type>>,
    ) -> Result<AnyValueEnum<'ctx>, Diagnostic<usize>> {
        match &value {
            Variant::Bool(value) => Ok(match self.to_llvm_type(r#type)? {
                AnyTypeEnum::IntType(t) => t.const_int(*value as u64, false).as_any_value_enum(),
                AnyTypeEnum::FloatType(t) => unsafe {
                    t.const_float_from_string(&value.to_string())
                        .as_any_value_enum()
                },
                _ => self
                    .context
                    .bool_type()
                    .const_int(*value as u64, false)
                    .as_any_value_enum(),
            }),
            Variant::Rational(value) => {
                let value = to_decimal(value).to_string();
                let dot_index = value.find(".").unwrap_or(value.len());
                Ok(match self.to_llvm_type(r#type)? {
                    AnyTypeEnum::FloatType(t) => {
                        unsafe { t.const_float_from_string(&value) }.as_any_value_enum()
                    }
                    AnyTypeEnum::IntType(t) => t
                        .const_int_from_string(&value[..dot_index], StringRadix::Decimal)
                        .unwrap()
                        .as_any_value_enum(),
                    _ => unsafe {
                        self.context
                            .f64_type()
                            .const_float_from_string(&value.to_string())
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
                Ok(match self.to_llvm_type(Rc::clone(&element_type))? {
                    AnyTypeEnum::ArrayType(t) => {
                        let mut values = Vec::new();
                        for value in array {
                            values.push(
                                self.to_llvm_value(value.clone(), Rc::clone(&element_type))?
                                    .into_array_value(),
                            );
                        }
                        t.const_array(&values).as_any_value_enum()
                    }
                    AnyTypeEnum::IntType(t) => {
                        let mut values = Vec::new();
                        for value in array {
                            values.push(
                                self.to_llvm_value(value.clone(), Rc::clone(&element_type))?
                                    .into_int_value(),
                            );
                        }
                        t.const_array(&values).as_any_value_enum()
                    }
                    AnyTypeEnum::FloatType(t) => {
                        let mut values = Vec::new();
                        for value in array {
                            values.push(
                                self.to_llvm_value(value.clone(), Rc::clone(&element_type))?
                                    .into_float_value(),
                            );
                        }
                        t.const_array(&values).as_any_value_enum()
                    }
                    AnyTypeEnum::PointerType(t) => {
                        let mut values = Vec::new();
                        for value in array {
                            values.push(
                                self.to_llvm_value(value.clone(), Rc::clone(&element_type))?
                                    .into_pointer_value(),
                            );
                        }
                        t.const_array(&values).as_any_value_enum()
                    }
                    AnyTypeEnum::ScalableVectorType(t) => {
                        let mut values = Vec::new();
                        for value in array {
                            values.push(
                                self.to_llvm_value(value.clone(), Rc::clone(&element_type))?
                                    .into_scalable_vector_value(),
                            );
                        }
                        t.const_array(&values).as_any_value_enum()
                    }
                    AnyTypeEnum::StructType(t) => {
                        let mut values = Vec::new();
                        for value in array {
                            values.push(
                                self.to_llvm_value(value.clone(), Rc::clone(&element_type))?
                                    .into_struct_value(),
                            );
                        }
                        t.const_array(&values).as_any_value_enum()
                    }
                    AnyTypeEnum::VectorType(t) => {
                        let mut values = Vec::new();
                        for value in array {
                            values.push(
                                self.to_llvm_value(value.clone(), Rc::clone(&element_type))?
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
                    TypeKind::Complex(Some(t)) => match self.to_llvm_type(Rc::clone(t))? {
                        AnyTypeEnum::FloatType(t) => t,
                        _ => self.context.f64_type(),
                    },
                    _ => self.context.f64_type(),
                };
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
            }
            Variant::Unknown => Err(Diagnostic::error()
                .with_message(format!("unkown value for type"))
                .with_label(Label::primary(
                    r#type.borrow().file_id,
                    r#type.borrow().span,
                ))),
        }
    }
}
