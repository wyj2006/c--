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
    Unkown {
        arguments: Option<Pair<'a, Rule>>,
    },
}

#[derive(Debug)]
pub struct Initializer<'a> {
    pub span: Span<'a>,
    pub designation: Vec<Designation<'a>>, //只有braced initializer中的initializer才有可能有
    pub kind: InitializerKind<'a>,
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

#[derive(Debug)]
pub enum InitializerKind<'a> {
    Braced(Vec<Rc<RefCell<Initializer<'a>>>>),
    Expr(Rc<RefCell<Expr<'a>>>),
}
