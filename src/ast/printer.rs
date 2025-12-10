use super::decl::{Declaration, DeclarationKind};
use super::expr::{Expr, ExprKind, GenericAssoc};
use super::stmt::{Stmt, StmtKind};
use super::{
    Attribute, AttributeKind, Designation, DesignationKind, Initializer, InitializerKind,
    TranslationUnit,
};
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

impl Print for TranslationUnit<'_> {
    fn display(&self) -> String {
        format!(
            "TranslationUnit <{:?},{:?}>",
            self.span.start_pos().line_col(),
            self.span.end_pos().line_col()
        )
    }

    fn children_display(&self, indent: usize) -> Vec<String> {
        let mut lines = Vec::new();
        for decl in &self.decls {
            lines.extend(decl.print_line(indent));
        }
        lines
    }
}

impl Print for Declaration<'_> {
    fn display(&self) -> String {
        format!(
            "{} <{:?},{:?}> {} {} {:?}",
            match &self.kind {
                DeclarationKind::Var { initializer: _ } => "VarDecl",
                DeclarationKind::Function {
                    parameter_decls: _,
                    function_specs,
                    body: _,
                } => &format!(
                    "FunctionDecl {:?}",
                    function_specs
                        .iter()
                        .map(|x| format!(
                            "{:?}<({:?}),({:?})>",
                            x.kind,
                            x.span.start_pos().line_col(),
                            x.span.end_pos().line_col()
                        ))
                        .collect::<Vec<String>>()
                        .join(" ")
                ),
                DeclarationKind::Type => "TypeDecl",
                DeclarationKind::Record { members_decl: _ } => "RecordDecl",
                DeclarationKind::Enum { enumerators: _ } => "EnumDecl",
                DeclarationKind::StaticAssert { expr: _ } => "StaticAssert",
                DeclarationKind::Attribute => "AttributeDecl",
                DeclarationKind::Enumerator { value: _ } => "Enumerator",
                DeclarationKind::Parameter => "ParamDecl",
                DeclarationKind::Member { bit_field: _ } => "MemberDecl",
            },
            self.span.start_pos().line_col(),
            self.span.end_pos().line_col(),
            self.name,
            self.r#type.borrow(),
            self.storage_classes
        )
    }

    fn children_display(&self, indent: usize) -> Vec<String> {
        let mut lines = Vec::new();

        lines.extend(self.attributes.print_line(indent));

        match &self.kind {
            DeclarationKind::Var { initializer } => {
                lines.extend(initializer.print_line(indent));
            }
            DeclarationKind::Function {
                parameter_decls,
                function_specs: _,
                body,
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

impl Print for Initializer<'_> {
    fn display(&self) -> String {
        format!(
            "Initializer <{:?},{:?}>",
            self.span.start_pos().line_col(),
            self.span.end_pos().line_col()
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

impl Print for Stmt<'_> {
    fn display(&self) -> String {
        format!(
            "{} <{:?},{:?}>",
            match &self.kind {
                StmtKind::Break => "Break",
                StmtKind::Case { .. } => "Case",
                StmtKind::Compound(_) => "Compound",
                StmtKind::Continue => "Continue",
                StmtKind::Decl(_) => "DeclStmt",
                StmtKind::Default(_) => "Default",
                StmtKind::DoWhile {
                    condition: _,
                    body: _,
                } => "DoWhile",
                StmtKind::Expr(_) => "ExprStmt",
                StmtKind::For { .. } => "For",
                StmtKind::Goto(name) => &format!("Goto {}", name),
                StmtKind::If { .. } => "If",
                StmtKind::Label { name, .. } => &format!("Label {}", name),
                StmtKind::Null => "Null",
                StmtKind::Return { .. } => "Return",
                StmtKind::Switch { .. } => "Switch",
                StmtKind::While { .. } => "While",
            },
            self.span.start_pos().line_col(),
            self.span.end_pos().line_col()
        )
    }

    fn children_display(&self, indent: usize) -> Vec<String> {
        let mut lines = Vec::new();

        match &self.kind {
            StmtKind::Case { expr, stmt } => {
                lines.extend(expr.print_line(indent));
                lines.extend(stmt.print_line(indent));
            }
            StmtKind::Compound(t) => lines.extend(t.print_line(indent)),
            StmtKind::Decl(t) => lines.extend(t.print_line(indent)),
            StmtKind::DoWhile { condition, body } => {
                lines.extend(condition.print_line(indent));
                lines.extend(body.print_line(indent));
            }
            StmtKind::Expr(t) => lines.extend(t.print_line(indent)),
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

impl Print for Expr<'_> {
    fn display(&self) -> String {
        format!(
            "{} <{:?},{:?}>",
            match &self.kind {
                ExprKind::Alignof(t) => &format!("Alignof {}", t.borrow()),
                ExprKind::BinOp { op, .. } => &format!("BinOp {:?}", op),
                ExprKind::Cast { r#type, .. } => &format!("Cast {}", r#type.borrow()),
                ExprKind::Char { prefix, raw_value } => &format!("Char {prefix:?} {raw_value}"),
                ExprKind::CompoundLiteral {
                    storage_classes,
                    r#type,
                    ..
                } => &format!("CompoundLiteral {:?} {}", storage_classes, r#type.borrow()),
                ExprKind::False => "False",
                ExprKind::Float {
                    base,
                    raw_value,
                    type_suffix,
                } => &format!("Float {base} {raw_value} {type_suffix:?}"),
                ExprKind::Integer {
                    base,
                    raw_value,
                    type_suffix,
                } => &format!("Integer {base} {raw_value} {type_suffix:?}"),
                ExprKind::MemberAccess {
                    through_pointer,
                    name,
                } => &format!(
                    "MemberAccess {} {}",
                    if *through_pointer { "->" } else { "." },
                    name
                ),
                ExprKind::Name(name) => &format!("Name {name}"),
                ExprKind::Nullptr => "Nullptr",
                ExprKind::SizeOf(t) => &format!("SizeOf {}", t.borrow()),
                ExprKind::String { prefix, raw_value } => &format!("String {prefix:?} {raw_value}"),
                ExprKind::True => "True",
                ExprKind::GenericSelection { .. } => "GenericSelection",
                ExprKind::UnaryOp { op, .. } => &format!("UnaryOp {:?}", op),
                ExprKind::Conditional { .. } => "Conditional",
                ExprKind::FunctionCall { .. } => "FunctionCall",
                ExprKind::Subscript { .. } => "Subscript",
            },
            self.span.start_pos().line_col(),
            self.span.end_pos().line_col()
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
            ExprKind::CompoundLiteral { initializer, .. } => {
                lines.extend(initializer.print_line(indent))
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
            _ => {}
        }

        lines
    }
}

impl Print for GenericAssoc<'_> {
    fn display(&self) -> String {
        format!(
            "GenericAssoc <{:?},{:?}> {}",
            self.span.start_pos().line_col(),
            self.span.end_pos().line_col(),
            if let Some(t) = &self.r#type {
                format!("{}", t.borrow())
            } else {
                "".to_string()
            }
        )
    }

    fn children_display(&self, indent: usize) -> Vec<String> {
        self.expr.print_line(indent)
    }
}

impl Print for Attribute<'_> {
    fn display(&self) -> String {
        format!(
            "{} <{:?},{:?}>",
            match &self.kind {
                AttributeKind::Unkown { arguments } => &format!(
                    "Attribute {}::{} ({:?})",
                    self.prefix_name.clone().unwrap_or("".to_string()),
                    self.name,
                    arguments
                ),
                AttributeKind::AlignAs { r#type, .. } => &format!(
                    "AlignAs {}",
                    if let Some(t) = &r#type {
                        format!("{}", t.borrow())
                    } else {
                        "".to_string()
                    }
                ),
            },
            self.span.start_pos().line_col(),
            self.span.end_pos().line_col(),
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

impl Print for Designation<'_> {
    fn display(&self) -> String {
        format!(
            "Designation <{:?},{:?}> {}",
            self.span.start_pos().line_col(),
            self.span.end_pos().line_col(),
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
