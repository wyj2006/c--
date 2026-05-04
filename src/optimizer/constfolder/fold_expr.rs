use crate::{
    ast::expr::{BinOpKind, Expr, ExprKind, UnaryOpKind},
    ctype::TypeKind,
    optimizer::constfolder::ConstFolder,
    symtab::SymbolKind,
    variant::Variant,
};
use codespan_reporting::diagnostic::Diagnostic;
use num::BigInt;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

impl ConstFolder {
    pub fn visit_expr(
        &self,
        node: Rc<RefCell<Expr>>,
        mut vars: HashMap<String, Variant>,
    ) -> Result<HashMap<String, Variant>, Diagnostic<usize>> {
        let mut value = Variant::Unknown;
        let mut node = node.borrow_mut();
        match &node.kind {
            ExprKind::Name(name) => {
                if let Some(symbol) = &node.symbol {
                    value = match &symbol.borrow().kind {
                        SymbolKind::EnumConst { value } => Variant::Int(value.clone()),
                        SymbolKind::Object { .. } => {
                            if let Some(t) = vars.get(name) {
                                t.clone()
                            } else {
                                value
                            }
                        }
                        _ => value,
                    }
                }
            }
            ExprKind::Cast { target, .. } => {
                vars = self.visit_expr(Rc::clone(target), vars)?;
                value = target.borrow().value.clone();
            }
            ExprKind::GenericSelection { assocs, .. } => {
                for assoc in assocs {
                    if assoc.borrow().is_selected {
                        vars = self.visit_expr(Rc::clone(&assoc.borrow().expr), vars)?;
                        value = assoc.borrow().expr.borrow().value.clone();
                        break;
                    }
                }
            }
            ExprKind::FunctionCall { target, arguments } => {
                vars = self.visit_expr(Rc::clone(target), vars)?;
                for argument in arguments {
                    vars = self.visit_expr(Rc::clone(argument), vars)?;
                }
                vars = self.prevent(&vars);
            }
            ExprKind::Subscript { target, index } => {
                vars = self.visit_expr(Rc::clone(target), vars)?;
                vars = self.visit_expr(Rc::clone(index), vars)?;

                let mut i = Variant::Unknown;
                match (
                    &target.borrow().r#type.borrow().kind,
                    &index.borrow().r#type.borrow().kind,
                ) {
                    (TypeKind::Pointer(_), x) if x.is_integer() => {
                        value = target.borrow().value.clone();
                        i = index.borrow().value.clone();
                    }
                    (x, TypeKind::Pointer(_)) if x.is_integer() => {
                        value = index.borrow().value.clone();
                        i = target.borrow().value.clone();
                    }
                    (TypeKind::Array { .. }, x) if x.is_integer() => {
                        value = target.borrow().value.clone();
                        i = index.borrow().value.clone();
                    }
                    (x, TypeKind::Array { .. }) if x.is_integer() => {
                        value = index.borrow().value.clone();
                        i = target.borrow().value.clone();
                    }
                    _ => {}
                }
                value = value.get(&i).clone();
            }
            ExprKind::MemberAccess { target, .. } => {
                vars = self.visit_expr(Rc::clone(target), vars)?;
            }
            ExprKind::UnaryOp { op, operand } => {
                vars = self.visit_expr(Rc::clone(operand), vars)?;
                match op {
                    UnaryOpKind::Not => value = !operand.borrow().value.clone(),
                    UnaryOpKind::Positive => value = operand.borrow().value.clone(),
                    UnaryOpKind::Negative => value = -operand.borrow().value.clone(),
                    UnaryOpKind::BitNot => value = !operand.borrow().value.clone(),
                    UnaryOpKind::PrefixInc | UnaryOpKind::PostfixInc => {
                        value = &operand.borrow().value + Variant::Int(BigInt::from(1));
                    }
                    UnaryOpKind::PrefixDec | UnaryOpKind::PostfixDec => {
                        value = &operand.borrow().value - Variant::Int(BigInt::from(1));
                    }
                    _ => {}
                }
            }
            ExprKind::BinOp { op, left, right } => {
                vars = self.visit_expr(Rc::clone(left), vars)?;
                vars = self.visit_expr(Rc::clone(right), vars)?;

                let x = &left.borrow().value;
                let y = &right.borrow().value;

                match op {
                    BinOpKind::And => value = x.and(y),
                    BinOpKind::Or => value = x.or(y),
                    BinOpKind::Lt => value = x.lt(y),
                    BinOpKind::Le => value = x.le(y),
                    BinOpKind::Gt => value = x.gt(y),
                    BinOpKind::Ge => value = x.ge(y),
                    BinOpKind::Eq => value = x.eq(y),
                    BinOpKind::Neq => value = x.neq(y),
                    BinOpKind::Add | BinOpKind::AddAssign => value = x + y,
                    BinOpKind::Sub | BinOpKind::SubAssign => value = x - y,
                    BinOpKind::Mul | BinOpKind::MulAssign => value = x * y,
                    BinOpKind::Div | BinOpKind::DivAssign => value = x / y,
                    BinOpKind::Mod | BinOpKind::ModAssign => value = x % y,
                    BinOpKind::BitAnd | BinOpKind::BitAndAssign => value = x & y,
                    BinOpKind::BitOr | BinOpKind::BitOrAssign => value = x | y,
                    BinOpKind::BitXOr | BinOpKind::BitXOrAssign => value = x ^ y,
                    BinOpKind::LShift | BinOpKind::LShiftAssign => value = x << y,
                    BinOpKind::RShift | BinOpKind::RShiftAssign => value = x >> y,
                    BinOpKind::Assign => value = y.clone(),
                    BinOpKind::Comma => value = y.clone(),
                }
            }
            ExprKind::Conditional {
                condition,
                true_expr,
                false_expr,
            } => {
                vars = self.visit_expr(Rc::clone(condition), vars)?;
                let a = self.visit_expr(Rc::clone(true_expr), vars.clone())?;
                let b = self.visit_expr(Rc::clone(false_expr), vars)?;

                vars = self.intersection(&a, &b);

                value = if let Some(t) = condition.borrow().value.bool() {
                    if t {
                        true_expr.borrow().value.clone()
                    } else {
                        false_expr.borrow().value.clone()
                    }
                } else {
                    value
                };
            }
            ExprKind::CompoundLiteral { initializer, .. } => {
                vars = self.visit_initializer(Rc::clone(&initializer), vars)?;
            }
            _ => value = node.value.clone(),
        }

        //处理赋值相关的语句
        match &*node {
            Expr {
                kind:
                    ExprKind::BinOp {
                        op:
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
                            | BinOpKind::BitXOrAssign,
                        ..
                    },
                symbol,
                ..
            }
            | Expr {
                kind:
                    ExprKind::UnaryOp {
                        op:
                            UnaryOpKind::PostfixDec
                            | UnaryOpKind::PostfixInc
                            | UnaryOpKind::PrefixDec
                            | UnaryOpKind::PrefixInc,
                        ..
                    },
                symbol,
                ..
            } => {
                if let Some(t) = symbol {
                    vars.insert(t.borrow().name.clone(), value.clone());
                } else {
                    //无法保证是否是别名, 所以保守一点
                    value = Variant::Unknown;
                    vars = self.prevent(&vars);
                }
            }
            _ => {}
        }

        node.value = value;

        Ok(vars)
    }
}
