use super::Initializer;
use crate::ast::decl::StorageClass;
use crate::ctype::Type;
use pest::Span;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct Expr<'a> {
    pub span: Span<'a>,
    pub kind: ExprKind<'a>,
}

#[derive(Debug)]
pub enum ExprKind<'a> {
    Name(String),
    Integer {
        base: usize,
        raw_value: String,
        type_suffix: Vec<String>,
    },
    Float {
        base: usize,
        raw_value: String,
        type_suffix: Vec<String>,
    },
    String {
        prefix: EncodePrefix,
        raw_value: String,
    },
    Char {
        prefix: EncodePrefix,
        raw_value: String,
    },
    True,
    False,
    Nullptr,
    GenericSelection {
        control_expr: Rc<RefCell<Expr<'a>>>,
        assocs: Vec<Rc<RefCell<GenericAssoc<'a>>>>,
    },
    CompoundLiteral {
        storage_classes: Vec<StorageClass<'a>>,
        r#type: Rc<RefCell<Type<'a>>>,
        initializer: Rc<RefCell<Initializer<'a>>>,
    },
    BinOp {
        op: BinOpKind,
        left: Rc<RefCell<Expr<'a>>>,
        right: Rc<RefCell<Expr<'a>>>,
    },
    UnaryOp {
        op: UnaryOpKind,
        operand: Rc<RefCell<Expr<'a>>>,
    },
    Cast {
        r#type: Rc<RefCell<Type<'a>>>,
        target: Rc<RefCell<Expr<'a>>>,
    },
    Subscript {
        target: Rc<RefCell<Expr<'a>>>,
        index: Rc<RefCell<Expr<'a>>>,
    },
    MemberAccess {
        through_pointer: bool,
        name: String,
    },
    FunctionCall {
        target: Rc<RefCell<Expr<'a>>>,
        arguments: Vec<Rc<RefCell<Expr<'a>>>>,
    },
    SizeOf(Rc<RefCell<Type<'a>>>), //这里sizeof的对象是类型, UnaryOp中的sizeof的对象是表达式
    Alignof(Rc<RefCell<Type<'a>>>),
    Conditional {
        condition: Rc<RefCell<Expr<'a>>>,
        true_expr: Rc<RefCell<Expr<'a>>>,
        false_expr: Rc<RefCell<Expr<'a>>>,
    },
}

#[derive(Debug)]
pub struct GenericAssoc<'a> {
    pub span: Span<'a>,
    pub r#type: Option<Rc<RefCell<Type<'a>>>>, //为None的就是default
    pub expr: Rc<RefCell<Expr<'a>>>,
}

#[derive(Debug)]
pub enum EncodePrefix {
    Default,
    UTF8,
    UTF16,
    UTF32,
    Wide,
}

#[derive(Debug)]
pub enum BinOpKind {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    LShift,
    RShift,
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Neq,
    BitAnd,
    BitXOr,
    BitOr,
    And,
    Or,
    Comma,
    Assign,
    MulAssign,
    DivAssign,
    ModAssign,
    AddAssign,
    SubAssign,
    LShiftAssign,
    RShiftAssign,
    BitAndAssign,
    BitOrAssign,
    BitXOrAssign,
}

#[derive(Debug)]
pub enum UnaryOpKind {
    Positive,
    Negative,
    BitNot,
    Not,
    Dereference,
    AddressOf,
    SizeOf,
    PrefixInc,
    PrefixDec,
    PostfixInc,
    PostfixDec,
}
