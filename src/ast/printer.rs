use super::decl::{Declaration, DeclarationKind};
use super::expr::{Expr, ExprKind, GenericAssoc};
use super::stmt::{Stmt, StmtKind};
use super::{
    Attribute, AttributeKind, Designation, DesignationKind, Initializer, InitializerKind,
    TranslationUnit,
};
use crate::files;
use crate::variant::Variant;
use codespan::Span;
use codespan_reporting::files::Files;
use std::{cell::RefCell, rc::Rc};

pub trait Print {
    fn print(&self) {
        println!("{}", self.print_line(0).join("\n"));
    }

    fn display(&self) -> String;
    fn children_display(&self, indent: usize) -> Vec<String>;

    fn print_line(&self, indent: usize) -> Vec<String> {
        let mut lines = Vec::new();
        let mut white_space = String::new();
        for _ in 0..indent {
            white_space += "  ";
        }
        lines.push(format!("{}{}", white_space, self.display()));
        for line in self.children_display(indent + 1) {
            lines.push(line)
        }
        lines
    }
}

impl<T: Print> Print for Rc<RefCell<T>> {
    fn display(&self) -> String {
        self.borrow().display()
    }

    fn children_display(&self, indent: usize) -> Vec<String> {
        self.borrow().children_display(indent)
    }
}

impl<T: Print> Print for Vec<T> {
    fn display(&self) -> String {
        unreachable!()
    }

    fn children_display(&self, _indent: usize) -> Vec<String> {
        unreachable!()
    }

    fn print_line(&self, indent: usize) -> Vec<String> {
        let mut lines = Vec::new();
        for i in self {
            lines.extend(i.print_line(indent));
        }
        lines
    }
}

impl<T: Print> Print for Option<T> {
    fn display(&self) -> String {
        unreachable!()
    }

    fn children_display(&self, _indent: usize) -> Vec<String> {
        unreachable!()
    }

    fn print_line(&self, indent: usize) -> Vec<String> {
        if let Some(t) = self {
            t.print_line(indent)
        } else {
            Vec::new()
        }
    }
}

impl Print for TranslationUnit {
    fn display(&self) -> String {
        format!(
            "TranslationUnit {}",
            format_location(self.file_id, self.span)
        )
    }

    fn children_display(&self, indent: usize) -> Vec<String> {
        let mut lines = Vec::new();
        lines.extend(self.decls.print_line(indent));
        lines
    }
}

impl Print for Declaration {
    fn display(&self) -> String {
        format!(
            "{} {} {} {} {}",
            match &self.kind {
                DeclarationKind::Var { initializer: _ } => "VarDecl",
                DeclarationKind::Function { function_specs, .. } => &format!(
                    "FunctionDecl {}",
                    function_specs
                        .iter()
                        .map(|x| x.kind.to_string())
                        .collect::<Vec<String>>()
                        .join(" ")
                ),
                DeclarationKind::Type => "TypeDecl",
                DeclarationKind::Record { .. } => "RecordDecl",
                DeclarationKind::Enum { .. } => "EnumDecl",
                DeclarationKind::StaticAssert { .. } => "StaticAssert",
                DeclarationKind::Attribute => "AttributeDecl",
                DeclarationKind::Enumerator { .. } => "Enumerator",
                DeclarationKind::Parameter => "ParamDecl",
                DeclarationKind::Member { .. } => "MemberDecl",
            },
            format_location(self.file_id, self.span),
            self.name,
            match &self.kind {
                //这些节点的type字段不会被用到
                DeclarationKind::Attribute
                | DeclarationKind::Enumerator { .. }
                | DeclarationKind::StaticAssert { .. } => "".to_string(),
                _ => self.r#type.borrow().to_string(),
            },
            self.storage_classes
                .iter()
                .map(|x| x.kind.to_string())
                .collect::<Vec<String>>()
                .join(" ")
        )
    }

    fn children_display(&self, indent: usize) -> Vec<String> {
        let mut lines = Vec::new();

        lines.extend(self.attributes.print_line(indent));
        lines.extend(self.children.print_line(indent));

        match &self.kind {
            DeclarationKind::Var { initializer } => {
                lines.extend(initializer.print_line(indent));
            }
            DeclarationKind::Function {
                parameter_decls,
                body,
                ..
            } => {
                lines.extend(parameter_decls.print_line(indent));
                lines.extend(body.print_line(indent));
            }
            DeclarationKind::Type => {}
            DeclarationKind::Record { members_decl } => {
                lines.extend(members_decl.print_line(indent));
            }
            DeclarationKind::Enum { enumerators } => {
                lines.extend(enumerators.print_line(indent));
            }
            DeclarationKind::StaticAssert { expr } => {
                lines.extend(expr.print_line(indent));
            }
            DeclarationKind::Attribute => {}
            DeclarationKind::Enumerator { value } => {
                lines.extend(value.print_line(indent));
            }
            DeclarationKind::Parameter => {}
            DeclarationKind::Member { bit_field } => {
                lines.extend(bit_field.print_line(indent));
            }
        }

        lines
    }
}

impl Print for Initializer {
    fn display(&self) -> String {
        format!(
            "Initializer {} {} {}",
            format_location(self.file_id, self.span),
            self.r#type.borrow(),
            self.value
        )
    }

    fn children_display(&self, indent: usize) -> Vec<String> {
        let mut lines = Vec::new();

        lines.extend(self.designation.print_line(indent));

        match &self.kind {
            InitializerKind::Braced(t) => lines.extend(t.print_line(indent)),
            InitializerKind::Expr(t) => lines.extend(t.print_line(indent)),
        }

        lines
    }
}

impl Print for Stmt {
    fn display(&self) -> String {
        format!(
            "{} {}",
            match &self.kind {
                StmtKind::Break => "Break",
                StmtKind::Case { .. } => "Case",
                StmtKind::Compound(_) => "Compound",
                StmtKind::Continue => "Continue",
                StmtKind::Default(_) => "Default",
                StmtKind::DoWhile {
                    condition: _,
                    body: _,
                } => "DoWhile",
                StmtKind::DeclExpr {
                    decls: Some(_),
                    expr: None,
                } => "DeclStmt",
                StmtKind::DeclExpr {
                    decls: None,
                    expr: Some(_),
                } => "ExprStmt",
                StmtKind::DeclExpr { .. } => "DeclExprStmt",
                StmtKind::For { .. } => "For",
                StmtKind::Goto(name) => &format!("Goto {}", name),
                StmtKind::If { .. } => "If",
                StmtKind::Label { name, .. } => &format!("Label {}", name),
                StmtKind::Null => "Null",
                StmtKind::Return { .. } => "Return",
                StmtKind::Switch { .. } => "Switch",
                StmtKind::While { .. } => "While",
            },
            format_location(self.file_id, self.span)
        )
    }

    fn children_display(&self, indent: usize) -> Vec<String> {
        let mut lines = Vec::new();

        lines.extend(self.attributes.print_line(indent));

        match &self.kind {
            StmtKind::Case { expr, stmt } => {
                lines.extend(expr.print_line(indent));
                lines.extend(stmt.print_line(indent));
            }
            StmtKind::Compound(t) => lines.extend(t.print_line(indent)),
            StmtKind::DoWhile { condition, body } => {
                lines.extend(condition.print_line(indent));
                lines.extend(body.print_line(indent));
            }
            StmtKind::DeclExpr { decls, expr } => {
                lines.extend(decls.print_line(indent));
                lines.extend(expr.print_line(indent));
            }
            StmtKind::For {
                init_expr,
                init_decl,
                condition,
                iter_expr,
                body,
            } => {
                lines.extend(init_expr.print_line(indent));
                lines.extend(init_decl.print_line(indent));
                lines.extend(condition.print_line(indent));
                lines.extend(iter_expr.print_line(indent));
                lines.extend(body.print_line(indent));
            }
            StmtKind::If {
                condition,
                body,
                else_body,
            } => {
                lines.extend(condition.print_line(indent));
                lines.extend(body.print_line(indent));
                lines.extend(else_body.print_line(indent));
            }
            StmtKind::Label { stmt, .. } => {
                lines.extend(stmt.print_line(indent));
            }
            StmtKind::Default(t) => {
                lines.extend(t.print_line(indent));
            }
            StmtKind::Return { expr } => lines.extend(expr.print_line(indent)),
            StmtKind::Switch { condition, body } => {
                lines.extend(condition.print_line(indent));
                lines.extend(body.print_line(indent));
            }
            StmtKind::While { condition, body } => {
                lines.extend(condition.print_line(indent));
                lines.extend(body.print_line(indent));
            }
            _ => {}
        }

        lines
    }
}

impl Print for Expr {
    fn display(&self) -> String {
        format!(
            "{} {} {} {} {}",
            match &self.kind {
                ExprKind::Alignof { r#type, .. } => &format!("Alignof {}", r#type.borrow()),
                ExprKind::BinOp { op, .. } => &format!("BinOp {:?}", op),
                ExprKind::Cast {
                    is_implicit: false,
                    method,
                    ..
                } => &format!("Cast <{method}>"),
                ExprKind::Cast {
                    is_implicit: true,
                    method,
                    ..
                } => &format!("ImplicitCast <{method}>"),
                ExprKind::Char { prefix, text } => &format!("Char {prefix:?} {text:?}"),
                ExprKind::CompoundLiteral {
                    storage_classes, ..
                } => &format!(
                    "CompoundLiteral {} {}",
                    storage_classes
                        .iter()
                        .map(|x| x.kind.to_string())
                        .collect::<Vec<String>>()
                        .join(" "),
                    self.r#type.borrow()
                ),
                ExprKind::False => "False",
                ExprKind::Float {
                    base,
                    digits,
                    exp_base,
                    exponent,
                    type_suffix,
                } => &format!("Float {base} {digits} {exp_base} {exponent} {type_suffix:?}"),
                ExprKind::Integer {
                    base,
                    text,
                    type_suffix,
                } => &format!("Integer {base} {text} {type_suffix:?}"),
                ExprKind::MemberAccess {
                    target: _,
                    is_arrow,
                    name,
                } => &format!(
                    "MemberAccess {} {}",
                    if *is_arrow { "->" } else { "." },
                    name
                ),
                ExprKind::Name(name) => &format!("Name {name}"),
                ExprKind::Nullptr => "Nullptr",
                ExprKind::SizeOf { r#type, .. } => &format!(
                    "SizeOf {}",
                    if let Some(r#type) = r#type {
                        r#type.borrow().to_string()
                    } else {
                        "".to_string()
                    }
                ),
                ExprKind::String { prefix, text } => &format!("String {prefix:?} {text:?}"),
                ExprKind::True => "True",
                ExprKind::GenericSelection { .. } => "GenericSelection",
                ExprKind::UnaryOp { op, .. } => &format!("UnaryOp {:?}", op),
                ExprKind::Conditional { .. } => "Conditional",
                ExprKind::FunctionCall { .. } => "FunctionCall",
                ExprKind::Subscript { .. } => "Subscript",
            },
            format_location(self.file_id, self.span),
            self.r#type.borrow().to_string(),
            if let Variant::Unknown = self.value {
                String::new()
            } else {
                self.value.to_string()
            },
            if self.is_lvalue { "lvalue" } else { "" }
        )
    }

    fn children_display(&self, indent: usize) -> Vec<String> {
        let mut lines = Vec::new();

        match &self.kind {
            ExprKind::BinOp { left, right, .. } => {
                lines.extend(left.print_line(indent));
                lines.extend(right.print_line(indent));
            }
            ExprKind::Cast { target, .. } => lines.extend(target.print_line(indent)),
            ExprKind::CompoundLiteral {
                initializer, decls, ..
            } => {
                lines.extend(decls.print_line(indent));
                lines.extend(initializer.print_line(indent));
            }
            ExprKind::Conditional {
                condition,
                true_expr,
                false_expr,
            } => {
                lines.extend(condition.print_line(indent));
                lines.extend(true_expr.print_line(indent));
                lines.extend(false_expr.print_line(indent));
            }
            ExprKind::FunctionCall { target, arguments } => {
                lines.extend(target.print_line(indent));
                lines.extend(arguments.print_line(indent));
            }
            ExprKind::GenericSelection {
                control_expr,
                assocs,
            } => {
                lines.extend(control_expr.print_line(indent));
                lines.extend(assocs.print_line(indent));
            }
            ExprKind::Subscript { target, index } => {
                lines.extend(target.print_line(indent));
                lines.extend(index.print_line(indent));
            }
            ExprKind::UnaryOp { operand, .. } => {
                lines.extend(operand.print_line(indent));
            }
            ExprKind::MemberAccess { target, .. } => {
                lines.extend(target.print_line(indent));
            }
            ExprKind::SizeOf {
                expr: Some(expr),
                decls,
                ..
            } => {
                lines.extend(expr.print_line(indent));
                lines.extend(decls.print_line(indent));
            }
            ExprKind::SizeOf { decls, .. } | ExprKind::Alignof { decls, .. } => {
                lines.extend(decls.print_line(indent));
            }
            _ => {}
        }

        lines
    }
}

impl Print for GenericAssoc {
    fn display(&self) -> String {
        format!(
            "GenericAssoc {} {} {}",
            format_location(self.file_id, self.span),
            if let Some(t) = &self.r#type {
                format!("{}", t.borrow())
            } else {
                "".to_string()
            },
            if self.is_selected { "selected" } else { "" }
        )
    }

    fn children_display(&self, indent: usize) -> Vec<String> {
        let mut lines = Vec::new();
        lines.extend(self.decls.print_line(indent));
        lines.extend(self.expr.print_line(indent));
        lines
    }
}

impl Print for Attribute {
    fn display(&self) -> String {
        format!(
            "{} {}",
            match &self.kind {
                AttributeKind::Unkown { arguments } => format!(
                    "Attribute {} ({:?})",
                    match &self.prefix_name {
                        Some(t) => format!("{t}::{}", self.name),
                        None => format!("{}", self.name),
                    },
                    arguments
                ),
                AttributeKind::AlignAs { r#type, .. } => format!(
                    "AlignAs {}",
                    if let Some(t) = &r#type {
                        format!("{}", t.borrow())
                    } else {
                        "".to_string()
                    }
                ),
                AttributeKind::PtrFromArray { array_type } =>
                    format!("ArrayPtr {}", array_type.borrow().to_string()),
                AttributeKind::Deprecated { reason } => format!(
                    "Deprecated {}",
                    if let Some(t) = reason {
                        t.to_string()
                    } else {
                        "".to_string()
                    }
                ),
                AttributeKind::FallThrough => format!("FallThrough"),
                AttributeKind::MaybeUnused => format!("MaybeUnused"),
                AttributeKind::Nodiscard { reason } => format!(
                    "Nodiscard {}",
                    if let Some(t) = reason {
                        t.to_string()
                    } else {
                        "".to_string()
                    }
                ),
                AttributeKind::Noreturn => format!("Noreturn"),
                AttributeKind::Unsequenced => format!("Unsequenced"),
                AttributeKind::Reproducible => format!("Reproduciable"),
            },
            format_location(self.file_id, self.span)
        )
    }

    fn children_display(&self, indent: usize) -> Vec<String> {
        let mut lines = Vec::new();

        match &self.kind {
            AttributeKind::AlignAs { r#type: _, expr } => lines.extend(expr.print_line(indent)),
            _ => {}
        }

        lines
    }
}

impl Print for Designation {
    fn display(&self) -> String {
        format!(
            "Designation {} {}",
            format_location(self.file_id, self.span),
            match &self.kind {
                DesignationKind::MemberAccess(name) => &name,
                _ => "",
            }
        )
    }

    fn children_display(&self, indent: usize) -> Vec<String> {
        let mut lines = Vec::new();

        match &self.kind {
            DesignationKind::Subscript(t) => lines.extend(t.print_line(indent)),
            _ => {}
        }

        lines
    }
}

pub fn format_location(file_id: usize, span: Span) -> String {
    let start_location = files
        .lock()
        .unwrap()
        .location(file_id, span.start().to_usize())
        .unwrap();
    let end_location = files
        .lock()
        .unwrap()
        .location(file_id, span.end().to_usize())
        .unwrap();
    format!(
        "<({}, {}), ({}, {})>",
        start_location.line_number,
        start_location.column_number,
        end_location.line_number,
        end_location.column_number
    )
}
