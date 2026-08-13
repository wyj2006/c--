use crate::{
    ast::{Initializer, InitializerKind},
    legalizer::riscv::Legalizer,
};
use codespan_reporting::diagnostic::Diagnostic;
use std::{cell::RefCell, rc::Rc};

impl Legalizer {
    pub fn visit_initializer(
        &mut self,
        node: &Rc<RefCell<Initializer>>,
    ) -> Result<(), Diagnostic<usize>> {
        let Initializer { kind, r#type, .. } = &*node.borrow();

        self.legalize_type(r#type)?;

        match kind {
            InitializerKind::Braced(initializers) => {
                for initializer in initializers {
                    self.visit_initializer(initializer)?;
                }
            }
            InitializerKind::Expr(expr) => self.visit_expr(expr)?,
        }

        Ok(())
    }
}
