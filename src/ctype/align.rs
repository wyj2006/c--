use crate::{
    ast::AttributeKind,
    ctype::{Type, TypeKind},
    diagnostic::{Diagnostic, DiagnosticKind},
    variant::Variant,
};
use num::ToPrimitive;

impl<'a> Type<'a> {
    pub fn has_alignas(&self) -> bool {
        for attribute in &self.attributes {
            match &attribute.borrow().kind {
                AttributeKind::AlignAs { .. } => return true,
                _ => {}
            }
        }
        false
    }

    pub fn align(&self) -> Result<Option<usize>, Diagnostic<'a>> {
        let mut align = self.kind.align();
        for attribute in &self.attributes {
            match &attribute.borrow().kind {
                AttributeKind::AlignAs {
                    r#type: Some(r#type),
                    expr: None,
                } => {
                    align = match (r#type.borrow().align()?, align) {
                        (Some(a), Some(b)) => Some(a.max(b)),
                        (Some(a), None) | (None, Some(a)) => Some(a),
                        (None, None) => align,
                    }
                }
                AttributeKind::AlignAs {
                    r#type: None,
                    expr: Some(expr),
                } => match &expr.borrow().value {
                    Variant::Int(value) => {
                        align = match (value.to_usize(), align) {
                            (Some(a), Some(b)) => Some(a.max(b)),
                            (Some(a), None) | (None, Some(a)) => Some(a),
                            (None, None) => align,
                        }
                    }
                    _ => {
                        return Err(Diagnostic {
                            span: expr.borrow().span,
                            kind: DiagnosticKind::Error,
                            message: format!("alignas require an integer constant or a type"),
                            notes: vec![],
                        });
                    }
                },
                _ => {}
            }
        }
        Ok(align)
    }
}

impl<'a> TypeKind<'a> {
    pub fn align(&self) -> Option<usize> {
        match self {
            TypeKind::Int => Some(4),
            _ => None,
        }
    }
}
