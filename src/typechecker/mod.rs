pub mod check_decl;
pub mod check_expr;
pub mod check_init;
pub mod check_stmt;
#[cfg(test)]
pub mod tests;

use crate::{
    ast::{
        InitializerKind, TranslationUnit, decl::DeclarationKind, expr::ExprKind, stmt::StmtKind,
    },
    ctype::Type,
    diagnostic::Error,
    symtab::SymbolTable,
};
use std::{cell::RefCell, rc::Rc};

pub struct TypeChecker<'a> {
    pub cur_symtab: Rc<RefCell<SymbolTable<'a>>>,
    //作用域所位于的函数作用域
    pub func_symtab: Vec<Rc<RefCell<SymbolTable<'a>>>>,
    //正在处理的record类型
    pub records: Vec<Rc<RefCell<Type<'a>>>>,
    //正在处理的enum类型
    pub enums: Vec<Rc<RefCell<Type<'a>>>>,
    //上下文信息, 实际上就是调用路径
    pub contexts: Vec<Context<'a>>,
}

pub enum Context<'a> {
    //使用XXXKind避免重复借用
    Expr(ExprKind<'a>),
    #[allow(unused)]
    Decl(DeclarationKind<'a>),
    #[allow(unused)]
    Stmt(StmtKind<'a>),
    Init(InitializerKind<'a>),
    Typeof,
}

impl<'a> TypeChecker<'a> {
    pub fn new(cur_symtab: Rc<RefCell<SymbolTable<'a>>>) -> TypeChecker<'a> {
        TypeChecker {
            cur_symtab,
            func_symtab: vec![],
            records: vec![],
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

    pub fn leave_scope(&mut self) {
        let parent_symtab;
        {
            let cur_symtab = self.cur_symtab.borrow();
            match &cur_symtab.parent {
                Some(t) => parent_symtab = Some(Rc::clone(t)),
                None => parent_symtab = None,
            };
        }
        let Some(parent_symtab) = parent_symtab else {
            return;
        };
        self.cur_symtab = Rc::clone(&parent_symtab);
    }

    pub fn check(&mut self, ast: Rc<RefCell<TranslationUnit<'a>>>) -> Result<(), Error<'a>> {
        self.visit_translation_unit(ast)
    }

    pub fn visit_translation_unit(
        &mut self,
        node: Rc<RefCell<TranslationUnit<'a>>>,
    ) -> Result<(), Error<'a>> {
        for decl in &node.borrow().decls {
            self.visit_declaration(Rc::clone(&decl))?;
        }
        Ok(())
    }
}
