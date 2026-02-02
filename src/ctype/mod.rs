pub mod align;
pub mod cast;
pub mod display;
pub mod group;
pub mod integer;
pub mod size;

use crate::ast::{Attribute, expr::Expr};
use crate::ctype::cast::{array_to_ptr, func_to_ptr};
use crate::symtab::{Symbol, SymbolKind};
use indexmap::IndexMap;
use pest::Span;
use std::cell::RefCell;
use std::fmt::Display;
use std::iter::zip;
use std::mem::discriminant;
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
    Unsigned, //等同于UInt
    Signed,   //等同于Int
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
    Complex(Option<Rc<RefCell<Type<'a>>>>),
    Decimal32,
    Decimal64,
    Decimal128,
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
        members: Option<IndexMap<String, Rc<RefCell<Symbol<'a>>>>>,
    },
    Enum {
        name: String,
        underlying: Rc<RefCell<Type<'a>>>,
        enum_consts: Option<IndexMap<String, Rc<RefCell<Symbol<'a>>>>>,
    },
    Array {
        has_static: bool,
        has_star: bool,
        element_type: Rc<RefCell<Type<'a>>>,
        len_expr: Option<Rc<RefCell<Expr<'a>>>>,
    },
    Pointer(Rc<RefCell<Type<'a>>>),
    Auto(Option<Rc<RefCell<Type<'a>>>>),
    Nullptr,
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeQual {
    Const,
    Restrict,
    Volatile,
    Atomic,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RecordKind {
    Struct,
    Union,
}

impl Display for RecordKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                RecordKind::Struct => "struct",
                RecordKind::Union => "union",
            }
        )
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

impl<'a> Type<'a> {
    pub fn can_modify(&self) -> bool {
        self.kind.can_modify()
    }

    pub fn is_complete(&self) -> bool {
        self.kind.is_complete()
    }
}

impl TypeKind<'_> {
    pub fn can_modify(&self) -> bool {
        match self {
            TypeKind::Qualified { qualifiers, .. } => !qualifiers.contains(&TypeQual::Const),
            _ => true,
        }
    }

    pub fn is_complete(&self) -> bool {
        match self.size() {
            Some(_) => true,
            None => false,
        }
    }
}

//当这个类型位于参数列表里时应当转换成什么类型
pub fn as_parameter_type<'a>(a: Rc<RefCell<Type<'a>>>) -> Rc<RefCell<Type<'a>>> {
    match &a.borrow().kind {
        TypeKind::Array { .. } => array_to_ptr(Rc::clone(&a)),
        TypeKind::Function { .. } => func_to_ptr(Rc::clone(&a)),
        TypeKind::Qualified { qualifiers, r#type } => Rc::new(RefCell::new(Type {
            span: a.borrow().span,
            attributes: a.borrow().attributes.clone(),
            kind: TypeKind::Qualified {
                qualifiers: qualifiers.clone(),
                r#type: as_parameter_type(Rc::clone(r#type)),
            },
        })),
        _ => Rc::clone(&a),
    }
}

pub fn is_compatible<'a>(a: Rc<RefCell<Type<'a>>>, b: Rc<RefCell<Type<'a>>>) -> bool {
    if Rc::ptr_eq(&a, &b) {
        return true;
    }
    match (&a.borrow().kind, &b.borrow().kind) {
        (_, TypeKind::Typedef { r#type, .. }) => {
            if let Some(t) = r#type {
                is_compatible(Rc::clone(&a), Rc::clone(t))
            } else {
                false
            }
        }
        (TypeKind::Typedef { r#type, .. }, _) => {
            if let Some(t) = r#type {
                is_compatible(Rc::clone(t), Rc::clone(&b))
            } else {
                return false;
            }
        }
        (_, TypeKind::Qualified { r#type, .. }) => is_compatible(Rc::clone(&a), Rc::clone(r#type)),
        (TypeKind::Qualified { r#type, .. }, _) => is_compatible(Rc::clone(r#type), Rc::clone(&b)),
        (TypeKind::Pointer(a_pointee), TypeKind::Pointer(b_pointee)) => {
            is_compatible(Rc::clone(a_pointee), Rc::clone(b_pointee))
        }
        (
            TypeKind::Array {
                element_type: a_element_type,
                len_expr: a_len_expr,
                ..
            },
            TypeKind::Array {
                element_type: b_element_type,
                len_expr: b_len_expr,
                ..
            },
        ) => {
            if !is_compatible(Rc::clone(a_element_type), Rc::clone(b_element_type)) {
                false
            } else if let Some(a_len_expr) = a_len_expr
                    && let Some(b_len_expr) = b_len_expr
                    //长度不相等
                    && a_len_expr.borrow().value!=b_len_expr.borrow().value
            {
                false
            } else {
                true
            }
        }
        (
            TypeKind::Record {
                name: a_name,
                kind: a_kind,
                members: a_members,
            },
            TypeKind::Record {
                name: b_name,
                kind: b_kind,
                members: b_members,
            },
        ) => {
            if a_kind != b_kind || a_name != b_name {
                return false;
            } else if let Some(a_members) = a_members
                && let Some(b_members) = b_members
            {
                if a_members.len() != b_members.len() {
                    return false;
                } else {
                    for (name, a_member) in a_members {
                        let Some(b_member) = b_members.get(name) else {
                            return false;
                        };
                        let a_member = a_member.borrow();
                        let b_member = b_member.borrow();
                        if !is_compatible(Rc::clone(&a_member.r#type), Rc::clone(&b_member.r#type))
                        {
                            return false;
                        }

                        if let RecordKind::Struct = a_kind
                            && a_members.get_index_of(name) != b_members.get_index_of(name)
                        {
                            return false;
                        }
                        match (&a_member.kind, &b_member.kind) {
                            (
                                SymbolKind::Member {
                                    bit_field: Some(a_bit_field),
                                },
                                SymbolKind::Member {
                                    bit_field: Some(b_bit_field),
                                },
                            ) if a_bit_field != b_bit_field => return false,
                            _ => {}
                        }
                    }
                    true
                }
            } else {
                true
            }
        }
        (
            TypeKind::Enum {
                name: a_name,
                enum_consts: a_enum_consts,
                ..
            },
            TypeKind::Enum {
                name: b_name,
                enum_consts: b_enum_consts,
                ..
            },
        ) => {
            if a_name != b_name {
                return false;
            } else if let Some(a_enum_consts) = a_enum_consts
                && let Some(b_enum_consts) = b_enum_consts
            {
                if a_enum_consts.len() != b_enum_consts.len() {
                    return false;
                } else {
                    for (name, a_enum_const) in a_enum_consts {
                        let Some(b_enum_const) = b_enum_consts.get(name) else {
                            return false;
                        };
                        let a_enum_const = a_enum_const.borrow();
                        let b_enum_const = b_enum_const.borrow();
                        match (&a_enum_const.kind, &b_enum_const.kind) {
                            (
                                SymbolKind::EnumConst { value: a_value },
                                SymbolKind::EnumConst { value: b_value },
                            ) if a_value != b_value => return false,
                            _ => {}
                        }
                    }
                    true
                }
            } else {
                true
            }
        }
        (_, TypeKind::Enum { underlying, .. }) => {
            is_compatible(Rc::clone(&a), Rc::clone(underlying))
        }
        (TypeKind::Enum { underlying, .. }, _) => {
            is_compatible(Rc::clone(underlying), Rc::clone(&b))
        }
        (
            TypeKind::Function {
                return_type: a_return_type,
                parameters_type: a_parameters_type,
                has_varparam: a_has_varparam,
            },
            TypeKind::Function {
                return_type: b_return_type,
                parameters_type: b_parameters_type,
                has_varparam: b_has_varparam,
            },
        ) => {
            if *a_has_varparam != *b_has_varparam {
                return false;
            } else if a_parameters_type.len() != b_parameters_type.len() {
                return false;
            } else if !is_compatible(Rc::clone(a_return_type), Rc::clone(b_return_type)) {
                return false;
            } else {
                for (a_parameter, b_parameter) in zip(a_parameters_type, b_parameters_type) {
                    if !is_compatible(
                        as_parameter_type(Rc::clone(a_parameter)),
                        as_parameter_type(Rc::clone(b_parameter)),
                    ) {
                        return false;
                    }
                }
                true
            }
        }
        (a, b) => discriminant(a) == discriminant(b),
    }
}

pub fn pointee(a: Rc<RefCell<Type>>) -> Option<Rc<RefCell<Type>>> {
    match &a.borrow().kind {
        TypeKind::Pointer(pointee) => Some(Rc::clone(pointee)),
        TypeKind::Qualified { r#type, .. } => pointee(Rc::clone(r#type)),
        _ => None,
    }
}

pub fn array_element(a: Rc<RefCell<Type>>) -> Option<Rc<RefCell<Type>>> {
    match &a.borrow().kind {
        TypeKind::Array { element_type, .. } => Some(Rc::clone(element_type)),
        TypeKind::Qualified { r#type, .. } => array_element(Rc::clone(r#type)),
        _ => None,
    }
}

pub fn arith_result_type<'a>(
    a: Rc<RefCell<Type<'a>>>,
    b: Rc<RefCell<Type<'a>>>,
) -> Rc<RefCell<Type<'a>>> {
    if a.borrow().is_complex() {
        Rc::clone(&a)
    } else if b.borrow().is_complex() {
        Rc::clone(&b)
    } else {
        Rc::clone(&a)
    }
}
