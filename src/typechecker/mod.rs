pub mod check_decl;
pub mod check_expr;
pub mod check_init;
pub mod check_stmt;
#[cfg(test)]
pub mod tests;

use codespan_reporting::diagnostic::Diagnostic;

use crate::{
    ast::{
        InitializerKind, TranslationUnit, decl::DeclarationKind, expr::ExprKind, stmt::StmtKind,
    },
    ctype::Type,
    symtab::SymbolTable,
};
use std::{cell::RefCell, rc::Rc};

pub struct TypeChecker {
    pub cur_symtab: Rc<RefCell<SymbolTable>>,
    //作用域所位于的函数作用域
    pub func_symtabs: Vec<Rc<RefCell<SymbolTable>>>,
    //正在处理的函数类型
    pub func_types: Vec<Rc<RefCell<Type>>>,
    //正在处理的record类型的成员
    pub member_symtabs: Vec<Rc<RefCell<SymbolTable>>>,
    //正在处理的enum类型
    pub enums: Vec<Rc<RefCell<Type>>>,
    //上下文信息, 实际上就是调用路径
    pub contexts: Vec<Context>,
}

pub enum Context {
    //使用XXXKind避免重复借用
    Expr(ExprKind),
    Decl(DeclarationKind),
    Stmt(StmtKind),
    Init(InitializerKind),
    Typeof,
}

impl TypeChecker {
    pub fn new(cur_symtab: Rc<RefCell<SymbolTable>>) -> TypeChecker {
        TypeChecker {
            cur_symtab,
            func_symtabs: vec![],
            func_types: vec![],
            member_symtabs: vec![],
            enums: vec![],
            contexts: vec![],
        }
    }

    pub fn enter_scope(&mut self) {
        let new_symtab = Rc::new(RefCell::new(SymbolTable::new()));
        {
            let mut cur_symtab = self.cur_symtab.borrow_mut();
            new_symtab.borrow_mut().parent = Some(Rc::clone(&self.cur_symtab));
            cur_symtab.children.push(Rc::clone(&new_symtab));
        }
        self.cur_symtab = new_symtab;
    }

    pub fn leave_scope(&mut self) -> Rc<RefCell<SymbolTable>> {
        let cur_symtab = Rc::clone(&self.cur_symtab);
        let parent_symtab;
        {
            let cur_symtab = self.cur_symtab.borrow();
            match &cur_symtab.parent {
                Some(t) => parent_symtab = Some(Rc::clone(t)),
                None => parent_symtab = None,
            };
        }
        if let Some(parent_symtab) = &parent_symtab {
            self.cur_symtab = Rc::clone(parent_symtab);
        };
        cur_symtab
    }

    pub fn check(&mut self, ast: Rc<RefCell<TranslationUnit>>) -> Result<(), Diagnostic<usize>> {
        self.visit_translation_unit(ast)
    }

    pub fn visit_translation_unit(
        &mut self,
        node: Rc<RefCell<TranslationUnit>>,
    ) -> Result<(), Diagnostic<usize>> {
        for decl in &node.borrow().decls {
            self.visit_declaration(Rc::clone(&decl))?;
        }
        Ok(())
    }
}
