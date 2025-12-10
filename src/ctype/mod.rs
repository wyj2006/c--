pub mod display;

use crate::ast::{Attribute, expr::Expr};
use pest::Span;
use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Type<'a> {
    pub span: Span<'a>,
    pub attributes: Vec<Rc<RefCell<Attribute<'a>>>>,
    pub kind: TypeKind<'a>,
}

#[derive(Debug, Clone)]
pub enum TypeKind<'a> {
    Void,
    Bool,
    Char,
    SignedChar,
    UnsignedChar,
    Unsigned,
    Signed,
    Short,
    UShort,
    Int,
    UInt,
    Long,
    ULong,
    LongLong,
    ULongLong,
    BitInt {
        unsigned: bool,
        width_expr: Rc<RefCell<Expr<'a>>>,
    },
    Float,
    Double,
    LongDouble,
    Complex,
    Decimal32,
    Decimal64,
    Decimal128,
    FloatComplex,
    DoubleComplex,
    LongDoubleComplex,
    Function {
        return_type: Rc<RefCell<Type<'a>>>,
        parameters_type: Vec<Rc<RefCell<Type<'a>>>>,
        has_varparam: bool,
    },
    Qualified {
        qualifiers: Vec<TypeQual>,
        r#type: Rc<RefCell<Type<'a>>>,
    },
    Atomic(Rc<RefCell<Type<'a>>>),
    Typedef {
        name: String,
        r#type: Option<Rc<RefCell<Type<'a>>>>,
    },
    Typeof {
        unqual: bool,
        expr: Option<Rc<RefCell<Expr<'a>>>>,
        r#type: Option<Rc<RefCell<Type<'a>>>>,
    },
    Record {
        name: String,
        kind: RecordKind,
        //TODO record成员
    },
    Enum {
        name: String,
        underlying: Rc<RefCell<Type<'a>>>,
        //TODO 枚举值
    },
    Array {
        has_static: bool,
        has_star: bool,
        element_type: Rc<RefCell<Type<'a>>>,
        len_expr: Option<Rc<RefCell<Expr<'a>>>>,
    },
    Pointer(Rc<RefCell<Type<'a>>>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeQual {
    Const,
    Restrict,
    Volatile,
    Atomic,
}

#[derive(Debug, Clone)]
pub enum RecordKind {
    Struct,
    Union,
}

impl Type<'_> {
    pub fn is_pointer(&self) -> bool {
        if let TypeKind::Pointer(_) = &self.kind {
            true
        } else if let TypeKind::Qualified {
            qualifiers: _,
            r#type,
        } = &self.kind
        {
            r#type.borrow().is_pointer()
        } else {
            false
        }
    }

    pub fn is_array(&self) -> bool {
        if let TypeKind::Array { .. } = &self.kind {
            true
        } else if let TypeKind::Qualified {
            qualifiers: _,
            r#type,
        } = &self.kind
        {
            r#type.borrow().is_array()
        } else {
            false
        }
    }

    pub fn is_function(&self) -> bool {
        if let TypeKind::Function { .. } = &self.kind {
            true
        } else if let TypeKind::Qualified {
            qualifiers: _,
            r#type,
        } = &self.kind
        {
            r#type.borrow().is_function()
        } else {
            false
        }
    }
}

impl Display for TypeQual {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TypeQual::Atomic => "_Atomic",
                TypeQual::Const => "const",
                TypeQual::Restrict => "restrict",
                TypeQual::Volatile => "volatile",
            }
        )
    }
}
