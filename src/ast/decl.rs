use super::{Attribute, Initializer};
use crate::ast::expr::Expr;
use crate::ast::stmt::Stmt;
use crate::ctype::Type;
use codespan::Span;
use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Declaration {
    pub file_id: usize,
    pub span: Span,
    pub attributes: Vec<Rc<RefCell<Attribute>>>,
    pub name: String,
    pub r#type: Rc<RefCell<Type>>,
    pub storage_classes: Vec<StorageClass>,
    pub kind: DeclarationKind,
    pub children: Vec<Rc<RefCell<Declaration>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StorageClass {
    pub file_id: usize,
    pub span: Span,
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
pub struct FunctionSpec {
    pub file_id: usize,
    pub span: Span,
    pub kind: FunctionSpecKind,
}

#[derive(Debug, Clone)]
pub enum FunctionSpecKind {
    Inline,
}

#[derive(Debug, Clone)]
pub enum DeclarationKind {
    Var {
        initializer: Option<Rc<RefCell<Initializer>>>,
    },
    Function {
        //可能包含了参数中的record或者enum的定义
        parameter_decls: Vec<Rc<RefCell<Declaration>>>,
        function_specs: Vec<FunctionSpec>,
        body: Option<Rc<RefCell<Stmt>>>, //为None是声明
    },
    Type,
    Record {
        members_decl: Option<Vec<Rc<RefCell<Declaration>>>>, //为None是声明
    },
    Enum {
        //借用Declaration处理枚举值
        enumerators: Option<Vec<Rc<RefCell<Declaration>>>>, //为None是声明
    },
    StaticAssert {
        //message复用name字段
        expr: Rc<RefCell<Expr>>,
    },
    Attribute,
    Enumerator {
        value: Option<Rc<RefCell<Expr>>>,
    },
    Parameter,
    Member {
        bit_field: Option<Rc<RefCell<Expr>>>,
    },
}

impl Display for StorageClassKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                StorageClassKind::Auto => "auto",
                StorageClassKind::Constexpr => "constexpr",
                StorageClassKind::Extern => "extern",
                StorageClassKind::Register => "register",
                StorageClassKind::Static => "static",
                StorageClassKind::ThreadLocal => "thread_local",
                StorageClassKind::Typedef => "typedef",
            }
        )
    }
}

impl Display for FunctionSpecKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                FunctionSpecKind::Inline => "inline",
            }
        )
    }
}
