pub mod builder;
pub mod gen_decl;
pub mod gen_expr;
pub mod gen_init;
pub mod gen_stmt;
#[cfg(test)]
pub mod tests;

use crate::{ast::TranslationUnit, codegen::builder::Builder};
use codespan_reporting::diagnostic::Diagnostic;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub struct CodeGen<B: Builder> {
    pub builder: B,
    /*不同对象对应的basicblock信息
    约定: 对于循环来说, Vec的第一个元素代表break需要跳到的基本块, 第二个元素代表continue需要跳到的基本块
     */
    pub basic_blocks: HashMap<usize, Vec<B::BasicBlock>>,
}

impl<B: Builder> CodeGen<B> {
    pub fn new(builder: B) -> CodeGen<B> {
        CodeGen {
            builder,
            basic_blocks: HashMap::new(),
        }
    }

    pub fn r#gen(&mut self, ast: Rc<RefCell<TranslationUnit>>) -> Result<(), Diagnostic<usize>> {
        let TranslationUnit {
            file_id,
            span,
            decls,
            symtab,
        } = &*ast.borrow();

        self.builder.append_context(*file_id, *span);

        if let Some(symtab) = symtab {
            self.builder.enter_scope(symtab);
        }

        for decl in decls {
            self.visit_declaration(decl)?;
        }

        if let Some(_) = symtab {
            self.builder.leave_scope();
        }

        self.builder.pop_context();

        Ok(())
    }
}
