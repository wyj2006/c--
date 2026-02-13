use super::Attribute;
use super::decl::Declaration;
use super::expr::Expr;
use crate::file_map::source_lookup;
use crate::symtab::SymbolTable;
use codespan::Span;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct Stmt {
    pub file_id: usize,
    pub span: Span,
    pub attributes: Vec<Rc<RefCell<Attribute>>>,
    pub symtab: Option<Rc<RefCell<SymbolTable>>>,
    pub kind: StmtKind,
}

impl Stmt {
    pub fn new(file_id: usize, span: Span) -> Stmt {
        let (file_id, span) = source_lookup(file_id, span);
        Stmt {
            file_id,
            span,
            attributes: vec![],
            symtab: None,
            kind: StmtKind::Null,
        }
    }
}

#[derive(Debug, Clone)]
pub enum StmtKind {
    Compound(Vec<Rc<RefCell<Stmt>>>),
    If {
        condition: Rc<RefCell<Expr>>,
        body: Rc<RefCell<Stmt>>,
        else_body: Option<Rc<RefCell<Stmt>>>,
    },
    Switch {
        condition: Rc<RefCell<Expr>>,
        body: Rc<RefCell<Stmt>>,
    },
    While {
        condition: Rc<RefCell<Expr>>,
        body: Rc<RefCell<Stmt>>,
    },
    DoWhile {
        condition: Rc<RefCell<Expr>>,
        body: Rc<RefCell<Stmt>>,
    },
    For {
        init_expr: Option<Rc<RefCell<Expr>>>,
        init_decl: Option<Rc<RefCell<Declaration>>>,
        condition: Option<Rc<RefCell<Expr>>>,
        iter_expr: Option<Rc<RefCell<Expr>>>,
        body: Rc<RefCell<Stmt>>,
    },
    Goto(String),
    Continue,
    Break,
    Return {
        expr: Option<Rc<RefCell<Expr>>>,
    },
    Label {
        name: String,
        stmt: Option<Rc<RefCell<Stmt>>>,
    },
    Case {
        expr: Rc<RefCell<Expr>>,
        stmt: Option<Rc<RefCell<Stmt>>>,
    },
    Default(Option<Rc<RefCell<Stmt>>>),
    DeclExpr {
        //类似于sizeof用于消歧义的结构
        decls: Option<Vec<Rc<RefCell<Declaration>>>>,
        expr: Option<Rc<RefCell<Expr>>>,
    },
    Null,
}
