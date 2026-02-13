use crate::{
    ast::decl::{Declaration, DeclarationKind},
    codegen::{CodeGen, any_to_basic_type},
    diagnostic::map_builder_err,
    symtab::{Namespace, SymbolKind},
};
use codespan_reporting::diagnostic::{Diagnostic, Label};
use inkwell::{types::BasicType, values::AnyValue};
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
                        None, //TODO 可能有其它链接方式
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
            //TODO 初始化
            //TODO storage classes
            DeclarationKind::Var { initializer: _ } => {
                let r#type = match any_to_basic_type(self.to_llvm_type(Rc::clone(&node.r#type))?) {
                    Some(t) => t,
                    None => {
                        return Err(Diagnostic::error()
                            .with_message(format!("{} is not basic type", node.r#type.borrow()))
                            .with_label(Label::primary(
                                node.r#type.borrow().file_id,
                                node.r#type.borrow().span,
                            )));
                    }
                };
                self.symbol_values.insert(
                    self.lookup(Namespace::Ordinary, &node.name)
                        .unwrap()
                        .as_ptr() as usize,
                    if self.func_values.len() == 0 {
                        self.module
                            .add_global(r#type, None, &node.name)
                            .as_any_value_enum()
                    } else {
                        map_builder_err(
                            node.file_id,
                            node.span,
                            self.builder.build_alloca(r#type, &node.name),
                        )?
                        .as_any_value_enum()
                    },
                );
            }
            DeclarationKind::Parameter => {
                let symbol = self.lookup(Namespace::Ordinary, &node.name).unwrap();
                let r#type = match any_to_basic_type(self.to_llvm_type(Rc::clone(&node.r#type))?) {
                    Some(t) => t,
                    None => {
                        return Err(Diagnostic::error()
                            .with_message(format!("{} is not basic type", node.r#type.borrow()))
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
            _ => {}
        }
        Ok(())
    }
}
