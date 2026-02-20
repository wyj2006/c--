use crate::{
    ast::{Designation, DesignationKind, Initializer, InitializerKind},
    codegen::CodeGen,
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
                    .to_llvm_type(Rc::clone(&layout.r#type))?
                    .into_int_type()
                    .const_int(value, false)
                    .as_basic_value_enum())
            } else {
                let mut value = self
                    .to_llvm_type(Rc::clone(&layout.r#type))?
                    .into_int_type()
                    .const_int(0, false);
                let file_id = layout.r#type.borrow().file_id;
                let span = layout.r#type.borrow().span;
                for (bitfield, width, offset) in bitfields {
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
                    value =
                        map_builder_err(file_id, span, self.builder.build_or(value, shl, "or"))?;
                }
                Ok(value.as_basic_value_enum())
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

            if fields.iter().all(|x| x.is_const()) {
                Ok(self
                    .context
                    .const_struct(&fields, false)
                    .as_basic_value_enum())
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
                    value = map_builder_err(
                        file_id,
                        span,
                        self.builder.build_insert_value(value, *field, i as u32, ""),
                    )?
                    .into_struct_value();
                }
                Ok(value.as_basic_value_enum())
            }
        } else {
            Ok(
                match BasicTypeEnum::try_from(self.to_llvm_type(Rc::clone(&layout.r#type))?) {
                    Ok(t) => t.const_zero(),
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

    pub fn visit_initializer(
        &self,
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
                    let value = match BasicValueEnum::try_from(
                        self.visit_initializer(Rc::clone(initializer))?,
                    ) {
                        Ok(t) => t,
                        Err(_) => {
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
