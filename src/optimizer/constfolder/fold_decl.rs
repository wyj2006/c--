use crate::{
    ast::decl::{Declaration, DeclarationKind},
    optimizer::constfolder::ConstFolder,
    variant::Variant,
};
use codespan_reporting::diagnostic::Diagnostic;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

impl ConstFolder {
    pub fn visit_declaration(
        &mut self,
        node: Rc<RefCell<Declaration>>,
        mut vars: HashMap<String, Variant>,
    ) -> Result<HashMap<String, Variant>, Diagnostic<usize>> {
        let node = node.borrow();
        match &node.kind {
            DeclarationKind::Var {
                initializer: Some(initializer),
            } => {
                vars = self.visit_initializer(Rc::clone(initializer), vars)?;
                if !node.r#type.borrow().is_volatile() {
                    vars.insert(node.name.clone(), initializer.borrow().value.clone());
                }
            }
            DeclarationKind::Function {
                body: Some(body),
                symtab: Some(symtab),
                ..
            } => {
                self.func_symtabs.push(Rc::clone(symtab));
                vars = self.visit_stmt(Rc::clone(body), vars)?;
                self.func_symtabs.pop();
            }
            //类型中的表达式在语义分析的时候就已经折叠了
            _ => {}
        }
        Ok(vars)
    }
}
