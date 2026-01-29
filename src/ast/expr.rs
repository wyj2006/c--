use super::Initializer;
use crate::ast::decl::StorageClass;
use crate::ctype::{Type, TypeKind};
use crate::symtab::Symbol;
use crate::variant::Variant;
use pest::Span;
use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Expr<'a> {
    pub span: Span<'a>,
    pub kind: ExprKind<'a>,
    pub r#type: Rc<RefCell<Type<'a>>>,
    pub value: Variant,
    pub is_lvalue: bool,
    pub symbol: Option<Rc<RefCell<Symbol<'a>>>>,
}

#[derive(Debug, Clone)]
pub enum ExprKind<'a> {
    Name(String),
    Integer {
        base: u32,
        text: String,
        type_suffix: Vec<String>, //后缀, 全为小写
    },
    Float {
        base: u32,
        digits: String,           //有效数字
        exp_base: u32,            //指数的底数
        exponent: String,         //指数
        type_suffix: Vec<String>, //后缀, 全为小写
    },
    String {
        prefix: EncodePrefix,
        text: String,
    },
    Char {
        prefix: EncodePrefix,
        text: String,
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
        is_implicit: bool,
        target: Rc<RefCell<Expr<'a>>>,
    },
    Subscript {
        target: Rc<RefCell<Expr<'a>>>,
        index: Rc<RefCell<Expr<'a>>>,
    },
    MemberAccess {
        target: Rc<RefCell<Expr<'a>>>,
        is_arrow: bool,
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
    pub is_selected: bool,
    pub r#type: Option<Rc<RefCell<Type<'a>>>>, //为None的就是default
    pub expr: Rc<RefCell<Expr<'a>>>,
}

#[derive(Debug, Clone)]
pub enum EncodePrefix {
    Default,
    UTF8,
    UTF16,
    UTF32,
    Wide,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, PartialEq)]
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

macro_rules! unparse_with_priority {
    ($a:expr,$b:expr) => {{
        let a = $a.borrow();
        let a_unparsed = a.unparse();
        if a.priority() < $b.priority() {
            format!("({a_unparsed})")
        } else {
            a_unparsed
        }
    }};
}

impl<'a> Expr<'a> {
    pub fn new(span: Span<'a>) -> Self {
        Expr {
            span,
            kind: ExprKind::Nullptr,
            r#type: Rc::new(RefCell::new(Type {
                span,
                attributes: vec![],
                kind: TypeKind::Error,
            })),
            value: Variant::default(),
            is_lvalue: false,
            symbol: None,
        }
    }

    pub fn priority(&self) -> usize {
        match &self.kind {
            ExprKind::Alignof(..) => 95,
            ExprKind::BinOp { op, .. } => match op {
                BinOpKind::Mul | BinOpKind::Div | BinOpKind::Mod => 90,
                BinOpKind::Add | BinOpKind::Sub => 85,
                BinOpKind::LShift | BinOpKind::RShift => 80,
                BinOpKind::Lt | BinOpKind::Le | BinOpKind::Gt | BinOpKind::Ge => 75,
                BinOpKind::Eq | BinOpKind::Neq => 70,
                BinOpKind::BitAnd => 65,
                BinOpKind::BitXOr => 60,
                BinOpKind::BitOr => 55,
                BinOpKind::And => 50,
                BinOpKind::Or => 45,
                BinOpKind::Comma => 40,
                BinOpKind::Assign
                | BinOpKind::MulAssign
                | BinOpKind::DivAssign
                | BinOpKind::ModAssign
                | BinOpKind::AddAssign
                | BinOpKind::SubAssign
                | BinOpKind::LShiftAssign
                | BinOpKind::RShiftAssign
                | BinOpKind::BitAndAssign
                | BinOpKind::BitOrAssign
                | BinOpKind::BitXOrAssign => 30,
            },
            ExprKind::Cast { .. } => 95,
            ExprKind::Char { .. } => 100,
            ExprKind::CompoundLiteral { .. } => 100,
            ExprKind::Conditional { .. } => 35,
            ExprKind::False => 100,
            ExprKind::Float { .. } => 100,
            ExprKind::FunctionCall { .. } => 95,
            ExprKind::GenericSelection { .. } => 100,
            ExprKind::Integer { .. } => 100,
            ExprKind::MemberAccess { .. } => 95,
            ExprKind::Name(..) => 100,
            ExprKind::Nullptr => 100,
            ExprKind::SizeOf(..) => 95,
            ExprKind::String { .. } => 100,
            ExprKind::Subscript { .. } => 95,
            ExprKind::True => 100,
            ExprKind::UnaryOp { .. } => 95,
        }
    }

    pub fn unparse(&self) -> String {
        match &self.kind {
            ExprKind::Alignof(r#type) => format!("alignof({})", r#type.borrow().to_string()),
            ExprKind::BinOp { op, left, right } => format!(
                "{}{op}{}",
                unparse_with_priority!(left, self),
                unparse_with_priority!(right, self),
            ),
            ExprKind::Cast { target, .. } => format!(
                "({}){}",
                self.r#type.borrow().to_string(),
                unparse_with_priority!(target, self),
            ),
            ExprKind::Char { prefix, text } => format!("{prefix}'{text}'"),
            ExprKind::CompoundLiteral {
                storage_classes,
                initializer,
            } => format!(
                "({storage_classes:?} {}){}",
                self.r#type.borrow().to_string(),
                initializer.borrow().unparse()
            ),
            ExprKind::Conditional {
                condition,
                true_expr,
                false_expr,
            } => format!(
                "{}?{}:{}",
                unparse_with_priority!(condition, self),
                unparse_with_priority!(true_expr, self),
                unparse_with_priority!(false_expr, self)
            ),
            ExprKind::False => "false".to_string(),
            ExprKind::Float {
                base,
                digits,
                exp_base,
                exponent,
                type_suffix,
            } => format!(
                "{}{digits}{}{exponent}{}",
                if *base == 16 { "0x" } else { "" },
                if *exp_base == 2 { "p" } else { "e" },
                type_suffix.join("")
            ),
            ExprKind::FunctionCall { target, arguments } => {
                format!("{}({})", unparse_with_priority!(target, self), {
                    let mut args = vec![];
                    for arg in arguments {
                        args.push(arg.borrow().unparse());
                    }
                    args.join(", ")
                })
            }
            ExprKind::GenericSelection {
                control_expr,
                assocs,
            } => format!("_Generic({},{})", control_expr.borrow().unparse(), {
                let mut assocs_str = vec![];
                for assoc in assocs {
                    let assoc = assoc.borrow();
                    assocs_str.push(format!(
                        "{}:{}",
                        if let Some(r#type) = &assoc.r#type {
                            r#type.borrow().to_string()
                        } else {
                            "default".to_string()
                        },
                        assoc.expr.borrow().unparse()
                    ));
                }
                assocs_str.join(", ")
            }),
            ExprKind::Integer {
                base,
                text,
                type_suffix,
            } => format!(
                "{}{text}{}",
                {
                    match base {
                        2 => "0b",
                        8 => "0",
                        16 => "0x",
                        _ => "",
                    }
                },
                type_suffix.join("")
            ),
            ExprKind::MemberAccess {
                target,
                is_arrow: through_pointer,
                name,
            } => format!(
                "{}{}{name}",
                unparse_with_priority!(target, self),
                if *through_pointer { "->" } else { "." }
            ),
            ExprKind::Name(name) => name.to_string(),
            ExprKind::Nullptr => "nullptr".to_string(),
            ExprKind::SizeOf(r#type) => format!("sizeof({})", r#type.borrow().to_string()),
            ExprKind::String { prefix, text } => format!("{prefix}\"{text}\""),
            ExprKind::Subscript { target, index } => format!(
                "{}[{}]",
                unparse_with_priority!(target, self),
                index.borrow().unparse()
            ),
            ExprKind::True => "true".to_string(),
            ExprKind::UnaryOp { op, operand } => {
                if op.is_postifx() {
                    format!("{}{op}", unparse_with_priority!(operand, self))
                } else {
                    format!("{op}{}", unparse_with_priority!(operand, self))
                }
            }
        }
    }
}

impl Display for BinOpKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BinOpKind::Add => "+",
                BinOpKind::Sub => "-",
                BinOpKind::Mul => "*",
                BinOpKind::Div => "/",
                BinOpKind::Mod => "%",
                BinOpKind::LShift => "<<",
                BinOpKind::RShift => ">>",
                BinOpKind::Lt => "<",
                BinOpKind::Le => "<=",
                BinOpKind::Gt => ">",
                BinOpKind::Ge => ">=",
                BinOpKind::Eq => "==",
                BinOpKind::Neq => "!=",
                BinOpKind::BitAnd => "&",
                BinOpKind::BitXOr => "^",
                BinOpKind::BitOr => "|",
                BinOpKind::And => "&&",
                BinOpKind::Or => "||",
                BinOpKind::Comma => ",",
                BinOpKind::Assign => "=",
                BinOpKind::MulAssign => "*=",
                BinOpKind::DivAssign => "/=",
                BinOpKind::ModAssign => "%=",
                BinOpKind::AddAssign => "+=",
                BinOpKind::SubAssign => "-=",
                BinOpKind::LShiftAssign => "<<=",
                BinOpKind::RShiftAssign => ">>=",
                BinOpKind::BitAndAssign => "&=",
                BinOpKind::BitOrAssign => "|=",
                BinOpKind::BitXOrAssign => "^=",
            }
        )
    }
}

impl Display for EncodePrefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                EncodePrefix::Default => "",
                EncodePrefix::UTF8 => "u8",
                EncodePrefix::UTF16 => "u",
                EncodePrefix::UTF32 => "U",
                EncodePrefix::Wide => "L",
            }
        )
    }
}

impl Display for UnaryOpKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                UnaryOpKind::Positive => "+",
                UnaryOpKind::Negative => "-",
                UnaryOpKind::BitNot => "~",
                UnaryOpKind::Not => "!",
                UnaryOpKind::Dereference => "*",
                UnaryOpKind::AddressOf => "&",
                UnaryOpKind::SizeOf => "sizeof ",
                UnaryOpKind::PrefixInc => "++",
                UnaryOpKind::PrefixDec => "--",
                UnaryOpKind::PostfixInc => "++",
                UnaryOpKind::PostfixDec => "--",
            }
        )
    }
}

impl UnaryOpKind {
    pub fn is_postifx(&self) -> bool {
        match self {
            UnaryOpKind::PostfixInc | UnaryOpKind::PostfixDec => true,
            _ => false,
        }
    }
}
