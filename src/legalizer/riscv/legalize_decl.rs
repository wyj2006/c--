use crate::{
    ast::decl::{Declaration, DeclarationKind},
    legalizer::riscv::Legalizer,
};
use codespan_reporting::diagnostic::Diagnostic;
use std::{cell::RefCell, rc::Rc};

impl Legalizer {
    pub fn visit_declaration(
        &mut self,
        node: &Rc<RefCell<Declaration>>,
    ) -> Result<(), Diagnostic<usize>> {
        let Declaration {
            r#type,
            kind,
            children,
            ..
        } = &*node.borrow();

        self.legalize_type(r#type)?;

        for child in children {
            self.visit_declaration(child)?;
        }

        match kind {
            DeclarationKind::Var {
                initializer: Some(initializer),
            } => self.visit_initializer(initializer)?,
            DeclarationKind::Function {
                parameter_decls,
                body,
                symtab,
                ..
            } => {
                if let Some(t) = symtab {
                    self.enter_scope(t.clone());
                }

                for decl in parameter_decls {
                    self.visit_declaration(decl)?;
                }

                if let Some(body) = body {
                    self.visit_stmt(body)?;
                }

                if let Some(_) = symtab {
                    self.leave_scope();
                }
            }
            DeclarationKind::Record {
                members_decl: Some(members_decl),
            } => {
                for decl in members_decl {
                    self.visit_declaration(decl)?;
                }
            }
            DeclarationKind::Enum {
                enumerators: Some(enumerators),
            } => {
                for decl in enumerators {
                    self.visit_declaration(decl)?;
                }
            }
            DeclarationKind::StaticAssert { expr } => self.visit_expr(expr)?,
            DeclarationKind::Enumerator { value: Some(value) } => self.visit_expr(value)?,
            DeclarationKind::Member {
                bit_field: Some(bit_field),
            } => self.visit_expr(bit_field)?,
            _ => {}
        }

        Ok(())
    }
}
