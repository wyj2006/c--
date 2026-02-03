pub mod decl;
pub mod expr;
pub mod printer;
pub mod stmt;

use crate::ast::decl::Declaration;
use crate::ctype::Type;
use codespan::Span;
use expr::Expr;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct TranslationUnit {
    pub file_id: usize,
    pub span: Span,
    pub decls: Vec<Rc<RefCell<Declaration>>>,
}

#[derive(Debug)]
pub struct Attribute {
    pub file_id: usize,
    pub span: Span,
    pub prefix_name: Option<String>,
    pub name: String,
    pub kind: AttributeKind,
}

#[derive(Debug)]
pub enum AttributeKind {
    AlignAs {
        r#type: Option<Rc<RefCell<Type>>>,
        expr: Option<Rc<RefCell<Expr>>>,
    },
    //用于函数参数中由数组转换过来的指针类型, 保留原来数组类型的信息供后续分析
    PtrFromArray {
        array_type: Rc<RefCell<Type>>,
    },
    Deprecated {
        reason: Option<String>,
    },
    FallThrough,
    Nodiscard {
        reason: Option<String>,
    },
    MaybeUnused,
    Noreturn,
    Unsequenced,
    Reproducible,
    Unkown {
        arguments: Option<String>,
    },
}

#[derive(Debug)]
pub struct Initializer {
    pub file_id: usize,
    pub span: Span,
    pub designation: Vec<Designation>, //只有braced initializer中的initializer才有可能有
    pub kind: InitializerKind,
    pub r#type: Rc<RefCell<Type>>,
}

#[derive(Debug)]
pub struct Designation {
    pub file_id: usize,
    pub span: Span,
    pub kind: DesignationKind,
}

#[derive(Debug)]
pub enum DesignationKind {
    Subscript(Rc<RefCell<Expr>>),
    MemberAccess(String),
}

#[derive(Debug, Clone)]
pub enum InitializerKind {
    Braced(Vec<Rc<RefCell<Initializer>>>),
    Expr(Rc<RefCell<Expr>>),
}

impl Initializer {
    pub fn unparse(&self) -> String {
        format!(
            "{}{}",
            if self.designation.len() > 0 {
                let mut s = String::new();
                for designation in &self.designation {
                    match &designation.kind {
                        DesignationKind::MemberAccess(name) => s += format!(".{name}").as_str(),
                        DesignationKind::Subscript(index) => {
                            s += format!("[{}]", index.borrow().unparse()).as_str()
                        }
                    }
                }
                s + "="
            } else {
                "".to_string()
            },
            match &self.kind {
                InitializerKind::Expr(expr) => expr.borrow().unparse(),
                InitializerKind::Braced(initializers) => format!("{{{}}}", {
                    let mut t = vec![];
                    for initializer in initializers {
                        t.push(initializer.borrow().unparse());
                    }
                    t.join(", ")
                }),
            }
        )
    }
}

pub fn has_c_attribute(prefix_name: Option<String>, name: String) -> isize {
    match (prefix_name.as_deref(), name.as_str()) {
        (None, "deprecated") => 201904,
        (None, "fallthrough") => 201904,
        (None, "maybe_unused") => 201904,
        (None, "nodiscard") => 202003,
        (None, "noreturn") => 202202,
        (None, "unsequenced") => 202207,
        (None, "reproducible") => 202207,
        _ => 0,
    }
}
