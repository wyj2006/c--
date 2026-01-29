pub mod decl;
pub mod expr;
pub mod printer;
pub mod stmt;

use crate::ctype::Type;
use crate::{ast::decl::Declaration, parser::Rule};
use expr::Expr;
use pest::{Span, iterators::Pair};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct TranslationUnit<'a> {
    pub span: Span<'a>,
    pub decls: Vec<Rc<RefCell<Declaration<'a>>>>,
}

#[derive(Debug)]
pub struct Attribute<'a> {
    pub span: Span<'a>,
    pub prefix_name: Option<String>,
    pub name: String,
    pub kind: AttributeKind<'a>,
}

#[derive(Debug)]
pub enum AttributeKind<'a> {
    AlignAs {
        r#type: Option<Rc<RefCell<Type<'a>>>>,
        expr: Option<Rc<RefCell<Expr<'a>>>>,
    },
    //用于函数参数中由数组转换过来的指针类型, 保留原来数组类型的信息供后续分析
    PtrFromArray {
        array_type: Rc<RefCell<Type<'a>>>,
    },
    Unkown {
        arguments: Option<Pair<'a, Rule>>,
    },
}

#[derive(Debug)]
pub struct Initializer<'a> {
    pub span: Span<'a>,
    pub designation: Vec<Designation<'a>>, //只有braced initializer中的initializer才有可能有
    pub kind: InitializerKind<'a>,
    pub r#type: Rc<RefCell<Type<'a>>>,
}

#[derive(Debug)]
pub struct Designation<'a> {
    pub span: Span<'a>,
    pub kind: DesignationKind<'a>,
}

#[derive(Debug)]
pub enum DesignationKind<'a> {
    Subscript(Rc<RefCell<Expr<'a>>>),
    MemberAccess(String),
}

#[derive(Debug, Clone)]
pub enum InitializerKind<'a> {
    Braced(Vec<Rc<RefCell<Initializer<'a>>>>),
    Expr(Rc<RefCell<Expr<'a>>>),
}

impl Initializer<'_> {
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
