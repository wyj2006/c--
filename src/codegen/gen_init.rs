use std::{cell::RefCell, collections::HashMap, rc::Rc};

use codespan_reporting::diagnostic::{Diagnostic, Label};

use crate::{
    ast::{Initializer, InitializerKind},
    codegen::{CodeGen, builder::Builder},
    ctype::layout::{ConstDesignation, compute_layout},
};

impl<B: Builder> CodeGen<B> {
    pub fn visit_initializer(
        &mut self,
        node: &Rc<RefCell<Initializer>>,
    ) -> Result<B::Value, Diagnostic<usize>> {
        let Initializer {
            file_id,
            span,
            designation: _,
            kind,
            r#type,
            value,
            has_side_effects,
        } = &*node.borrow();

        self.builder.append_context(*file_id, *span);

        if !*has_side_effects {
            if let Ok(t) = self.builder.variant_to_value(value, r#type) {
                self.builder.pop_context();
                return Ok(t);
            }
        }

        let result;
        match kind {
            InitializerKind::Braced(initializers) => {
                let mut init_values = HashMap::new();
                for (i, initializer) in initializers.iter().enumerate() {
                    if i > 0 && (r#type.borrow().is_union() || r#type.borrow().is_scale()) {
                        break;
                    }
                    let designation =
                        ConstDesignation::from_designation(&initializer.borrow().designation)?;
                    let value = self.visit_initializer(initializer)?;
                    init_values.insert(designation, value);
                }

                let layout = match compute_layout(Rc::clone(&r#type)) {
                    Some(t) => t,
                    None => {
                        return Err(Diagnostic::error()
                            .with_message(format!(
                                "cannot compute layout for '{}'",
                                r#type.borrow()
                            ))
                            .with_label(Label::primary(*file_id, *span)));
                    }
                };

                result = self.builder.layout_to_value(layout, vec![], &init_values)?;
            }
            InitializerKind::Expr(expr) => result = self.visit_expr(expr)?,
        }

        self.builder.pop_context();

        Ok(result)
    }
}
