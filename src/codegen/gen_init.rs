use crate::{
    ast::{Designation, DesignationKind, Initializer, InitializerKind},
    codegen::{CodeGen, any_to_basic_type, any_to_basic_value},
    ctype::layout::{ConstDesignation, Layout, compute_layout},
    diagnostic::map_builder_err,
    variant::Variant,
};
use codespan_reporting::diagnostic::{Diagnostic, Label};
use inkwell::{
    types::BasicTypeEnum,
    values::{AnyValue, AnyValueEnum, BasicValue, BasicValueEnum},
};
use num::ToPrimitive;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

impl ConstDesignation {
    pub fn from_designation(
        designations: &Vec<Designation>,
    ) -> Result<Vec<ConstDesignation>, Diagnostic<usize>> {
        let mut result = vec![];
        for designation in designations {
            result.push(match &designation.kind {
                DesignationKind::Subscript(t) => {
                    ConstDesignation::Subscript(match &t.borrow().value {
                        Variant::Int(t) => match t.to_usize() {
                            Some(t) => t,
                            None => Err(Diagnostic::error()
                                .with_message(format!("invalid integer: {t}"))
                                .with_label(Label::primary(
                                    designation.file_id,
                                    designation.span,
                                )))?,
                        },
                        _ => Err(Diagnostic::error()
                            .with_message(format!("subscript must be an integer"))
                            .with_label(Label::primary(designation.file_id, designation.span)))?,
                    })
                }
                DesignationKind::MemberAccess(t) => ConstDesignation::MemberAccess(t.clone()),
            });
        }
        Ok(result)
    }
}

impl<'ctx> CodeGen<'ctx> {
    pub fn layout_to_value(
        &self,
        layout: Layout,
        path: Vec<ConstDesignation>,
        init_values: &HashMap<Vec<ConstDesignation>, BasicValueEnum<'ctx>>,
    ) -> Result<BasicValueEnum<'ctx>, Diagnostic<usize>> {
        if let Some(t) = init_values.get(&path) {
            Ok(*t)
        }
        //不是Record或者数组却有children, 说明是位域
        else if !(layout.r#type.borrow().is_array() || layout.r#type.borrow().is_record())
            && layout.children.len() > 0
        {
            let mut values = vec![];
            for child in layout.children {
                let mut path = path.clone();
                if let Some(t) = &child.designation {
                    path.push(t.clone());
                }
                if let Some(t) = init_values.get(&path) {
                    values.push((t.into_int_value(), child.width, child.offset));
                }
            }
            if values.iter().all(|x| x.0.is_const()) {
                let mut value = 0;
                for (bitfield, width, offset) in values {
                    value |= (bitfield.get_zero_extended_constant().unwrap() & ((1 << width) - 1))
                        << offset;
                }
                Ok(self
                    .to_llvm_type(Rc::clone(&layout.r#type))?
                    .into_int_type()
                    .const_int(value, false)
                    .as_basic_value_enum())
            } else {
                let ty = self
                    .to_llvm_type(Rc::clone(&layout.r#type))?
                    .into_int_type();
                let file_id = layout.r#type.borrow().file_id;
                let span = layout.r#type.borrow().span;
                let ptr = map_builder_err(file_id, span, self.builder.build_alloca(ty, "init"))?;
                map_builder_err(
                    file_id,
                    span,
                    self.builder
                        .build_store(ptr, self.context.i32_type().const_zero()),
                )?;
                for (bitfield, width, offset) in values {
                    let mask = (1 << width) - 1;
                    let and = map_builder_err(
                        file_id,
                        span,
                        self.builder.build_and(
                            bitfield,
                            self.context.i32_type().const_int(mask, false),
                            "and",
                        ),
                    )?;
                    let shl = map_builder_err(
                        file_id,
                        span,
                        self.builder.build_left_shift(
                            and,
                            self.context.i32_type().const_int(offset as u64, false),
                            "shl",
                        ),
                    )?;
                    let or = map_builder_err(
                        file_id,
                        span,
                        self.builder.build_or(
                            map_builder_err(file_id, span, self.builder.build_load(ty, ptr, ""))?
                                .into_int_value(),
                            shl,
                            "or",
                        ),
                    )?;
                    map_builder_err(file_id, span, self.builder.build_store(ptr, or))?;
                }
                Ok(map_builder_err(
                    file_id,
                    span,
                    self.builder.build_load(ty, ptr, ""),
                )?)
            }
        } else if layout.r#type.borrow().is_array() || layout.r#type.borrow().is_record() {
            let mut values = vec![];
            'outer: for child in layout.children {
                let mut path = path.clone();
                if let Some(t) = &child.designation {
                    path.push(t.clone());
                }
                if layout.r#type.borrow().is_union() {
                    for designation in init_values.keys() {
                        if designation.starts_with(&path) {
                            values.push(self.layout_to_value(child.clone(), path, init_values)?);
                            break 'outer;
                        }
                    }
                } else {
                    values.push(self.layout_to_value(child.clone(), path, init_values)?);
                }
            }
            if values.iter().all(|x| x.is_const()) {
                Ok(self
                    .context
                    .const_struct(&values, false)
                    .as_basic_value_enum())
            } else {
                let ty = self.context.struct_type(
                    &values
                        .iter()
                        .map(|x| x.get_type())
                        .collect::<Vec<BasicTypeEnum>>(),
                    false,
                );
                let file_id = layout.r#type.borrow().file_id;
                let span = layout.r#type.borrow().span;
                let ptr = map_builder_err(file_id, span, self.builder.build_alloca(ty, ""))?;
                for (i, value) in values.iter().enumerate() {
                    let field_ptr = map_builder_err(
                        file_id,
                        span,
                        self.builder.build_struct_gep(ty, ptr, i as u32, ""),
                    )?;
                    map_builder_err(file_id, span, self.builder.build_store(field_ptr, *value))?;
                }
                Ok(map_builder_err(
                    file_id,
                    span,
                    self.builder.build_load(ty, ptr, ""),
                )?)
            }
        } else {
            Ok(
                match any_to_basic_type(self.to_llvm_type(Rc::clone(&layout.r#type))?) {
                    Some(t) => t.const_zero(),
                    None => {
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

    pub fn visit_initializer(
        &mut self,
        node: Rc<RefCell<Initializer>>,
    ) -> Result<AnyValueEnum<'ctx>, Diagnostic<usize>> {
        match &node.borrow().kind {
            InitializerKind::Braced(initializers) => {
                let mut init_values = HashMap::new();
                for (i, initializer) in initializers.iter().enumerate() {
                    if i > 0
                        && (node.borrow().r#type.borrow().is_union()
                            || node.borrow().r#type.borrow().is_scale())
                    {
                        break;
                    }
                    let designation =
                        ConstDesignation::from_designation(&initializer.borrow().designation)?;
                    let value =
                        match any_to_basic_value(self.visit_initializer(Rc::clone(initializer))?) {
                            Some(t) => t,
                            None => {
                                return Err(Diagnostic::error()
                                    .with_message("not a basic value")
                                    .with_label(Label::primary(
                                        initializer.borrow().file_id,
                                        initializer.borrow().span,
                                    )));
                            }
                        };
                    init_values.insert(designation, value);
                }

                let layout = match compute_layout(Rc::clone(&node.borrow().r#type)) {
                    Some(t) => t,
                    None => {
                        return Err(Diagnostic::error()
                            .with_message(format!(
                                "cannot compute layout for '{}'",
                                node.borrow().r#type.borrow()
                            ))
                            .with_label(Label::primary(
                                node.borrow().file_id,
                                node.borrow().span,
                            )));
                    }
                };

                Ok(self
                    .layout_to_value(layout, vec![], &init_values)?
                    .as_any_value_enum())
            }
            InitializerKind::Expr(expr) => Ok(self.visit_expr(Rc::clone(expr))?),
        }
    }
}
