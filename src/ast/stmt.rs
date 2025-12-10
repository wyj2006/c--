use super::Attribute;
use super::decl::Declaration;
use super::expr::Expr;
use pest::Span;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct Stmt<'a> {
    pub span: Span<'a>,
    pub attributes: Vec<Rc<RefCell<Attribute<'a>>>>,
    pub kind: StmtKind<'a>,
}

#[derive(Debug)]
pub enum StmtKind<'a> {
    Compound(Vec<Rc<RefCell<Stmt<'a>>>>),
    If {
        condition: Rc<RefCell<Expr<'a>>>,
        body: Rc<RefCell<Stmt<'a>>>,
        else_body: Option<Rc<RefCell<Stmt<'a>>>>,
    },
    Switch {
        condition: Rc<RefCell<Expr<'a>>>,
        body: Rc<RefCell<Stmt<'a>>>,
    },
    While {
        condition: Rc<RefCell<Expr<'a>>>,
        body: Rc<RefCell<Stmt<'a>>>,
    },
    DoWhile {
        condition: Rc<RefCell<Expr<'a>>>,
        body: Rc<RefCell<Stmt<'a>>>,
    },
    For {
        init_expr: Option<Rc<RefCell<Expr<'a>>>>,
        init_decl: Option<Rc<RefCell<Declaration<'a>>>>,
        condition: Option<Rc<RefCell<Expr<'a>>>>,
        iter_expr: Option<Rc<RefCell<Expr<'a>>>>,
        body: Rc<RefCell<Stmt<'a>>>,
    },
    Goto(String),
    Continue,
    Break,
    Return {
        expr: Option<Rc<RefCell<Expr<'a>>>>,
    },
    Expr(Rc<RefCell<Expr<'a>>>),
    Label {
        name: String,
        stmt: Option<Rc<RefCell<Stmt<'a>>>>,
    },
    Case {
        expr: Rc<RefCell<Expr<'a>>>,
        stmt: Option<Rc<RefCell<Stmt<'a>>>>,
    },
    Default(Option<Rc<RefCell<Stmt<'a>>>>),
    Decl(Rc<RefCell<Declaration<'a>>>),
    Null,
}
