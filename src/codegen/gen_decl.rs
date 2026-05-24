use crate::{
    ast::decl::{Declaration, DeclarationKind},
    codegen::{CodeGen, builder::Builder},
};
use codespan_reporting::diagnostic::Diagnostic;
use std::{cell::RefCell, rc::Rc};

impl<B: Builder> CodeGen<B> {
    pub fn visit_declaration(
        &mut self,
        node: &Rc<RefCell<Declaration>>,
    ) -> Result<(), Diagnostic<usize>> {
        let Declaration {
            file_id,
            span,
            attributes: _,
            name,
            r#type,
            storage_classes,
            kind,
            children: _,
        } = &*node.borrow();

        if name.len() == 0 {
            return Ok(());
        }

        self.builder.append_context(*file_id, *span);

        match kind {
            DeclarationKind::Var { initializer } => {
                let init_value = match initializer {
                    Some(t) => Some(self.visit_initializer(t)?),
                    None => None,
                };
                let vla_size = if r#type.borrow().is_vla() {
                    let len_expr = r#type.borrow().array_len_expr().unwrap();
                    Some(self.visit_expr(&len_expr)?)
                } else {
                    None
                };
                self.builder
                    .decl_variable(name, r#type, storage_classes, init_value, vla_size)?;
            }
            DeclarationKind::Function {
                parameter_decls,
                function_specs,
                body,
                symtab,
            } => {
                if let Some(symtab) = symtab {
                    self.builder.enter_scope(symtab);
                }

                let function =
                    self.builder
                        .decl_function(name, r#type, storage_classes, function_specs)?;

                if let Some(body) = body {
                    self.builder.enter_function(&function);

                    for decl in parameter_decls {
                        self.visit_declaration(decl)?;
                    }
                    self.visit_stmt(body)?;

                    self.builder.leave_function(&function)?;
                }

                if let Some(_) = symtab {
                    self.builder.leave_scope();
                }
            }
            DeclarationKind::Parameter => {
                self.builder.decl_parameter(name, r#type)?;
            }
            DeclarationKind::Record {
                members_decl: Some(_),
            } => {
                self.builder.decl_record(name, r#type)?;
            }
            _ => {}
        }

        self.builder.pop_context();

        Ok(())
    }
}
