use crate::{
    ast::decl::{Declaration, DeclarationKind, FunctionSpecKind, StorageClassKind},
    codegen::CodeGen,
    ctype::{RecordKind, TypeKind, array_element, layout::compute_layout},
    diagnostic::map_builder_err,
    symtab::{Namespace, SymbolKind},
};
use codespan_reporting::diagnostic::{Diagnostic, Label};
use inkwell::{
    module::Linkage,
    types::{AnyType, AnyTypeEnum, BasicType, BasicTypeEnum},
    values::{AnyValue, BasicValue, BasicValueEnum},
};
use std::{cell::RefCell, rc::Rc};

impl<'ctx> CodeGen<'ctx> {
    pub fn visit_declaration(
        &mut self,
        node: Rc<RefCell<Declaration>>,
    ) -> Result<(), Diagnostic<usize>> {
        let node = node.borrow();
        if node.name.len() == 0 {
            return Ok(());
        }
        match &node.kind {
            DeclarationKind::Function {
                parameter_decls,
                body,
                symtab,
                function_specs,
                ..
            } => {
                if let Some(symtab) = symtab {
                    self.enter_scope(Rc::clone(symtab));
                }

                let symbol = self.lookup(Namespace::Ordinary, &node.name).unwrap();
                let function = if let Some(t) = self.symbol_values.get(&(symbol.as_ptr() as usize))
                {
                    t.into_function_value()
                } else {
                    let function = self.module.add_function(
                        &node.name,
                        self.to_llvm_type(Rc::clone(&node.r#type))?
                            .into_function_type(),
                        if function_specs
                            .iter()
                            .any(|x| x.kind == FunctionSpecKind::Inline)
                            || node
                                .storage_classes
                                .iter()
                                .any(|x| x.kind == StorageClassKind::Static)
                        {
                            Some(Linkage::Internal)
                        } else if node
                            .storage_classes
                            .iter()
                            .any(|x| x.kind == StorageClassKind::Extern)
                        {
                            Some(Linkage::External)
                        } else {
                            Some(Linkage::External)
                        },
                    );
                    self.symbol_values
                        .insert(symbol.as_ptr() as usize, function.as_any_value_enum());
                    function
                };

                if let Some(body) = body {
                    let basic_block = self.context.append_basic_block(function, "entry");
                    self.builder.position_at_end(basic_block);

                    self.func_values.push(function);
                    for decl in parameter_decls {
                        self.visit_declaration(Rc::clone(decl))?;
                    }
                    self.visit_stmt(Rc::clone(body))?;
                    self.func_values.pop();

                    let mut last_block = basic_block;
                    while let Some(t) = last_block.get_next_basic_block() {
                        if t.get_name().to_str().unwrap().starts_with("unreach") {
                            unsafe { t.delete() }.unwrap();
                            continue;
                        }
                        last_block = t;
                    }
                    if let None = last_block.get_terminator() {
                        if let Some(t) = function.get_type().get_return_type() {
                            let retval = map_builder_err(
                                node.file_id,
                                node.span,
                                self.builder.build_alloca(t, ""),
                            )?;

                            map_builder_err(
                                node.file_id,
                                node.span,
                                self.builder.build_memset(
                                    retval,
                                    1,
                                    self.context.i8_type().const_int(0, false),
                                    t.size_of().unwrap(),
                                ),
                            )?;

                            map_builder_err(
                                node.file_id,
                                node.span,
                                self.builder.build_return(Some(&retval)),
                            )?;
                        } else {
                            map_builder_err(
                                node.file_id,
                                node.span,
                                self.builder.build_return(None),
                            )?;
                        }
                    }
                }

                if let Some(_) = symtab {
                    self.leave_scope();
                }
            }
            DeclarationKind::Var { initializer } => {
                let r#type =
                    match BasicTypeEnum::try_from(self.to_llvm_type(Rc::clone(&node.r#type))?) {
                        Ok(t) => t,
                        Err(_) => {
                            return Err(Diagnostic::error()
                                .with_message(format!(
                                    "{} is not a basic type",
                                    node.r#type.borrow()
                                ))
                                .with_label(Label::primary(
                                    node.r#type.borrow().file_id,
                                    node.r#type.borrow().span,
                                )));
                        }
                    };
                let init_value = match initializer {
                    Some(initializer) => Some(
                        match BasicValueEnum::try_from(
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
                        },
                    ),
                    None => None,
                };

                let value = if self.func_values.len() == 0
                    || node.storage_classes.iter().any(|x| {
                        x.kind == StorageClassKind::Static || x.kind == StorageClassKind::Extern
                    }) {
                    let value = self.module.add_global(r#type, None, &node.name);

                    value.set_thread_local(
                        node.storage_classes
                            .iter()
                            .any(|x| x.kind == StorageClassKind::ThreadLocal),
                    );

                    if node
                        .storage_classes
                        .iter()
                        .any(|x| x.kind == StorageClassKind::Static)
                    {
                        value.set_linkage(Linkage::Internal);
                    } else if node
                        .storage_classes
                        .iter()
                        .any(|x| x.kind == StorageClassKind::Extern)
                    {
                        value.set_linkage(Linkage::External);
                    }

                    if let Some(init_value) = init_value {
                        if init_value.is_const() {
                            value.set_initializer(&init_value);
                        } else {
                            return Err(Diagnostic::error()
                                .with_message("not a constant")
                                .with_label(Label::primary(
                                    initializer.as_ref().unwrap().borrow().file_id,
                                    initializer.as_ref().unwrap().borrow().span,
                                )));
                        }
                    }
                    value.as_any_value_enum()
                } else if node.r#type.borrow().is_vla() {
                    let len_expr = node.r#type.borrow().array_len_expr().unwrap();
                    let size = self.visit_expr(Rc::clone(&len_expr))?.into_int_value();
                    let element_type = array_element(Rc::clone(&node.r#type)).unwrap();
                    map_builder_err(
                        node.file_id,
                        node.span,
                        self.builder.build_array_alloca(
                            match BasicTypeEnum::try_from(
                                self.to_llvm_type(Rc::clone(&element_type))?,
                            ) {
                                Ok(t) => t,
                                Err(_) => {
                                    return Err(Diagnostic::error()
                                        .with_message(format!(
                                            "array element not has a basic type: '{}'",
                                            element_type.borrow()
                                        ))
                                        .with_label(Label::primary(
                                            element_type.borrow().file_id,
                                            element_type.borrow().span,
                                        )));
                                }
                            },
                            size,
                            &node.name,
                        ),
                    )?
                    .as_any_value_enum()
                } else {
                    let value = {
                        map_builder_err(
                            node.file_id,
                            node.span,
                            self.builder.build_alloca(r#type, &node.name),
                        )?
                    };
                    if let Some(init_value) = init_value {
                        map_builder_err(
                            node.file_id,
                            node.span,
                            self.builder.build_store(value, init_value),
                        )?;
                    }
                    value.as_any_value_enum()
                };
                self.symbol_values.insert(
                    self.lookup(Namespace::Ordinary, &node.name)
                        .unwrap()
                        .as_ptr() as usize,
                    value,
                );
            }
            DeclarationKind::Parameter => {
                let symbol = self.lookup(Namespace::Ordinary, &node.name).unwrap();
                let r#type =
                    match BasicTypeEnum::try_from(self.to_llvm_type(Rc::clone(&node.r#type))?) {
                        Ok(t) => t,
                        Err(_) => {
                            return Err(Diagnostic::error()
                                .with_message(format!(
                                    "{} is not a basic type",
                                    node.r#type.borrow()
                                ))
                                .with_label(Label::primary(
                                    node.r#type.borrow().file_id,
                                    node.r#type.borrow().span,
                                )));
                        }
                    };
                match &symbol.borrow().kind {
                    SymbolKind::Parameter { index, .. } => {
                        let param_value = map_builder_err(
                            node.file_id,
                            node.span,
                            self.builder.build_alloca(r#type, &node.name),
                        )?;
                        map_builder_err(
                            node.file_id,
                            node.span,
                            self.builder.build_store(
                                param_value,
                                self.func_values
                                    .last()
                                    .unwrap()
                                    .get_nth_param(*index)
                                    .unwrap(),
                            ),
                        )?;
                        self.symbol_values
                            .insert(symbol.as_ptr() as usize, param_value.as_any_value_enum());
                    }
                    _ => unreachable!(),
                }
            }
            DeclarationKind::Record {
                members_decl: Some(_),
            } => {
                let record_type = match &node.r#type.borrow().kind {
                    TypeKind::Record {
                        kind: RecordKind::Struct,
                        members: Some(_),
                        ..
                    } => {
                        let Some(layout) = compute_layout(Rc::clone(&node.r#type)) else {
                            return Err(Diagnostic::error()
                                .with_message(format!(
                                    "cannot compute layout for '{}'",
                                    node.r#type.borrow()
                                ))
                                .with_label(Label::primary(
                                    node.r#type.borrow().file_id,
                                    node.r#type.borrow().span,
                                )));
                        };
                        let mut field_types = Vec::new();
                        for child_layout in layout.children {
                            let child_layout_ty = &child_layout.r#type;
                            let llvm_member_ty = self.to_llvm_type(Rc::clone(child_layout_ty))?;
                            field_types.push(match llvm_member_ty {
                                AnyTypeEnum::ArrayType(_) => {
                                    llvm_member_ty.into_array_type().into()
                                }
                                AnyTypeEnum::FloatType(_) => {
                                    llvm_member_ty.into_float_type().into()
                                }
                                AnyTypeEnum::IntType(_) => llvm_member_ty.into_int_type().into(),
                                AnyTypeEnum::PointerType(_) => {
                                    llvm_member_ty.into_pointer_type().into()
                                }
                                AnyTypeEnum::ScalableVectorType(_) => {
                                    llvm_member_ty.into_scalable_vector_type().into()
                                }
                                AnyTypeEnum::StructType(_) => {
                                    llvm_member_ty.into_struct_type().into()
                                }
                                AnyTypeEnum::VectorType(_) => {
                                    llvm_member_ty.into_vector_type().into()
                                }
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
                                    (node.r#type.borrow().size().unwrap() * 8) as u32,
                                )
                                .as_basic_type_enum()],
                            false,
                        )
                        .as_any_type_enum(),
                    _ => unreachable!(),
                };
                self.record_types.insert(
                    self.lookup(Namespace::Tag, &node.name).unwrap().as_ptr() as usize,
                    record_type,
                );
            }
            _ => {}
        }
        Ok(())
    }
}
