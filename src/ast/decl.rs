use super::{Attribute, Initializer};
use crate::ast::expr::Expr;
use crate::ast::stmt::Stmt;
use crate::ctype::Type;
use pest::Span;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Declaration<'a> {
    pub span: Span<'a>,
    pub attributes: Vec<Rc<RefCell<Attribute<'a>>>>,
    pub name: String,
    pub r#type: Rc<RefCell<Type<'a>>>,
    pub storage_classes: Vec<StorageClass<'a>>,
    pub kind: DeclarationKind<'a>,
    pub children: Vec<Rc<RefCell<Declaration<'a>>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StorageClass<'a> {
    pub span: Span<'a>,
    pub kind: StorageClassKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StorageClassKind {
    Auto,
    Constexpr,
    Extern,
    Register,
    Static,
    ThreadLocal,
    Typedef,
}

#[derive(Debug, Clone)]
pub struct FunctionSpec<'a> {
    pub span: Span<'a>,
    pub kind: FunctionSpecKind,
}

#[derive(Debug, Clone)]
pub enum FunctionSpecKind {
    Inline,
    Noreturn,
}

#[derive(Debug, Clone)]
pub enum DeclarationKind<'a> {
    Var {
        initializer: Option<Rc<RefCell<Initializer<'a>>>>,
    },
    Function {
        //可能包含了参数中的record或者enum的定义
        parameter_decls: Vec<Rc<RefCell<Declaration<'a>>>>,
        function_specs: Vec<FunctionSpec<'a>>,
        body: Option<Rc<RefCell<Stmt<'a>>>>, //为None是声明
    },
    Type,
    Record {
        members_decl: Option<Vec<Rc<RefCell<Declaration<'a>>>>>, //为None是声明
    },
    Enum {
        //借用Declaration处理枚举值
        enumerators: Option<Vec<Rc<RefCell<Declaration<'a>>>>>, //为None是声明
    },
    StaticAssert {
        //message复用name字段
        expr: Rc<RefCell<Expr<'a>>>,
    },
    Attribute,
    Enumerator {
        value: Option<Rc<RefCell<Expr<'a>>>>,
    },
    Parameter,
    Member {
        bit_field: Option<Rc<RefCell<Expr<'a>>>>,
    },
}
