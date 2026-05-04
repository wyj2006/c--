use crate::{
    ast::{Initializer, InitializerKind},
    ctype::layout::{ConstDesignation, Layout, compute_layout},
    optimizer::constfolder::ConstFolder,
    variant::Variant,
};
use codespan_reporting::diagnostic::{Diagnostic, Label};
use num::BigInt;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

impl ConstFolder {
    pub fn layout_to_variant(
        &self,
        layout: Layout,
        path: Vec<ConstDesignation>,
        init_values: &HashMap<Vec<ConstDesignation>, Variant>,
    ) -> Result<Variant, Diagnostic<usize>> {
        if let Some(t) = init_values.get(&path) {
            Ok(t.clone())
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
                    bitfields.push((t, child.width, child.offset));
                }
            }

            let mut value = Variant::Int(BigInt::ZERO);
            for (bitfield, width, offset) in bitfields {
                value = value
                    | ((bitfield & Variant::Int(BigInt::from((1 << width) - 1)))
                        << Variant::Int(BigInt::from(offset)));
            }

            Ok(value)
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
                            fields.push(self.layout_to_variant(
                                child.clone(),
                                path,
                                init_values,
                            )?);
                            break 'outer;
                        }
                    }
                } else {
                    fields.push(self.layout_to_variant(child.clone(), path, init_values)?);
                }
            }

            Ok(Variant::Array(fields))
        } else {
            Ok(Variant::Int(BigInt::ZERO))
        }
    }

    pub fn visit_initializer(
        &self,
        node: Rc<RefCell<Initializer>>,
        mut vars: HashMap<String, Variant>,
    ) -> Result<HashMap<String, Variant>, Diagnostic<usize>> {
        let mut node = node.borrow_mut();
        match &node.kind {
            InitializerKind::Braced(initializers) => {
                let mut init_values = HashMap::new();
                for (i, initializer) in initializers.iter().enumerate() {
                    if i > 0 && (node.r#type.borrow().is_union() || node.r#type.borrow().is_scale())
                    {
                        break;
                    }
                    let designation =
                        ConstDesignation::from_designation(&initializer.borrow().designation)?;
                    vars = self.visit_initializer(Rc::clone(initializer), vars)?;
                    let value = initializer.borrow().value.clone();
                    init_values.insert(designation, value);
                }

                let layout = match compute_layout(Rc::clone(&node.r#type)) {
                    Some(t) => t,
                    None => {
                        return Err(Diagnostic::error()
                            .with_message(format!(
                                "cannot compute layout for '{}'",
                                node.r#type.borrow()
                            ))
                            .with_label(Label::primary(node.file_id, node.span)));
                    }
                };

                node.value = self.layout_to_variant(layout, vec![], &init_values)?;
            }
            InitializerKind::Expr(expr) => {
                vars = self.visit_expr(Rc::clone(expr), vars)?;
                let value = expr.borrow().value.clone();
                node.value = value;
            }
        }
        Ok(vars)
    }
}
