use super::{CParser, Rule};
use crate::{
    ast::expr::{BinOpKind, EncodePrefix, Expr, ExprKind, GenericAssoc, UnaryOpKind},
    ctype::{Type, TypeKind},
};
use pest::{
    error::Error,
    iterators::Pair,
    pratt_parser::{Assoc, Op, PrattParser},
};
use std::cell::RefCell;
use std::sync::LazyLock;
use std::{char, rc::Rc};

static PRATT_PARSER: LazyLock<PrattParser<Rule>> = LazyLock::new(|| {
    PrattParser::new()
        .op(Op::infix(Rule::comma, Assoc::Left))
        .op(Op::infix(Rule::or, Assoc::Left))
        .op(Op::infix(Rule::and, Assoc::Left))
        .op(Op::infix(Rule::bit_or, Assoc::Left))
        .op(Op::infix(Rule::bit_xor, Assoc::Left))
        .op(Op::infix(Rule::bit_and, Assoc::Left))
        .op(Op::infix(Rule::eq, Assoc::Left) | Op::infix(Rule::neq, Assoc::Left))
        .op(Op::infix(Rule::lt, Assoc::Left)
            | Op::infix(Rule::le, Assoc::Left)
            | Op::infix(Rule::gt, Assoc::Left)
            | Op::infix(Rule::ge, Assoc::Left))
        .op(Op::infix(Rule::lshift, Assoc::Left) | Op::infix(Rule::rshift, Assoc::Left))
        .op(Op::infix(Rule::add, Assoc::Left) | Op::infix(Rule::sub, Assoc::Left))
        .op(Op::infix(Rule::mul, Assoc::Left)
            | Op::infix(Rule::div, Assoc::Left)
            | Op::infix(Rule::r#mod, Assoc::Left))
        .op(Op::prefix(Rule::positve)
            | Op::prefix(Rule::negative)
            | Op::prefix(Rule::bit_not)
            | Op::prefix(Rule::not)
            | Op::prefix(Rule::dereference)
            | Op::prefix(Rule::addressof)
            | Op::prefix(Rule::cast)
            | Op::prefix(Rule::sizeof)
            | Op::prefix(Rule::prefix_increment)
            | Op::prefix(Rule::prefix_decrement))
        .op(Op::postfix(Rule::subscript)
            | Op::postfix(Rule::function_call)
            | Op::postfix(Rule::member_access)
            | Op::postfix(Rule::postfix_increment)
            | Op::postfix(Rule::postfix_decrement))
});

static FLOAT_SUFFIX: &[&str] = &[
    &"df", &"dd", &"dl", &"DF", &"DD", &"DL", &"f", &"l", &"F", &"L",
];
static INTEGER_SUFFIX: &[&str] = &[&"wb", &"WB", &"ll", &"LL", &"l", &"L", &"u", &"U"];

impl<'a> CParser<'a> {
    pub fn parse_assignment_expression(
        &self,
        rule: Pair<'a, Rule>,
    ) -> Result<Rc<RefCell<Expr<'a>>>, Error<Rule>> {
        let span = rule.as_span();
        let mut left = None;
        let mut op = None;
        let mut right = None;
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::conditional_expression => return self.parse_conditional_expression(rule),
                Rule::unary_expression => left = Some(self.parse_unary_expression(rule)?),
                Rule::assignment_operator => {
                    op = Some(match rule.as_str() {
                        "=" => BinOpKind::Assign,
                        "*=" => BinOpKind::MulAssign,
                        "/=" => BinOpKind::DivAssign,
                        "%=" => BinOpKind::ModAssign,
                        "+=" => BinOpKind::AddAssign,
                        "-=" => BinOpKind::SubAssign,
                        "<<=" => BinOpKind::LShiftAssign,
                        ">>=" => BinOpKind::RShiftAssign,
                        "&=" => BinOpKind::BitAndAssign,
                        "|==" => BinOpKind::BitOrAssign,
                        "^=" => BinOpKind::BitXOrAssign,
                        _ => unreachable!(),
                    })
                }
                Rule::assignment_expression => {
                    right = Some(self.parse_assignment_expression(rule)?)
                }
                _ => unreachable!(),
            }
        }
        let op = op.unwrap();
        let left = left.unwrap();
        let right = right.unwrap();
        Ok(Rc::new(RefCell::new(Expr {
            kind: ExprKind::BinOp { op, left, right },
            ..Expr::new(span)
        })))
    }

    pub fn parse_expression(
        &self,
        rule: Pair<'a, Rule>,
    ) -> Result<Rc<RefCell<Expr<'a>>>, Error<Rule>> {
        let mut expr = None;
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::assignment_expression => {
                    let span = rule.as_span();
                    let right = self.parse_assignment_expression(rule)?;
                    if let Some(left) = expr {
                        expr = Some(Rc::new(RefCell::new(Expr {
                            kind: ExprKind::BinOp {
                                op: BinOpKind::Comma,
                                left,
                                right,
                            },
                            is_lvalue: false,
                            ..Expr::new(span)
                        })));
                    } else {
                        expr = Some(right);
                    }
                }
                _ => unreachable!(),
            }
        }
        Ok(expr.unwrap())
    }

    pub fn parse_constant_expression(
        &self,
        rule: Pair<'a, Rule>,
    ) -> Result<Rc<RefCell<Expr<'a>>>, Error<Rule>> {
        self.parse_conditional_expression(rule)
    }

    pub fn parse_conditional_expression(
        &self,
        rule: Pair<'a, Rule>,
    ) -> Result<Rc<RefCell<Expr<'a>>>, Error<Rule>> {
        let span = rule.as_span();
        let mut exprs = Vec::new();
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::logical_OR_expression => exprs.push(self.parse_logical_or_expression(rule)?),
                Rule::expression => exprs.push(self.parse_expression(rule)?),
                Rule::conditional_expression => {
                    exprs.push(self.parse_conditional_expression(rule)?)
                }
                _ => unreachable!(),
            }
        }
        exprs.reverse();
        if exprs.len() == 1 {
            Ok(exprs.pop().unwrap())
        } else {
            Ok(Rc::new(RefCell::new(Expr {
                kind: ExprKind::Conditional {
                    condition: exprs.pop().unwrap(),
                    true_expr: exprs.pop().unwrap(),
                    false_expr: exprs.pop().unwrap(),
                },
                ..Expr::new(span)
            })))
        }
    }

    pub fn parse_unary_expression(
        &self,
        rule: Pair<'a, Rule>,
    ) -> Result<Rc<RefCell<Expr<'a>>>, Error<Rule>> {
        if rule.as_str().starts_with("sizeof") {
            let span = rule.as_span();
            let mut r#type = None;
            for rule in rule.into_inner() {
                match rule.as_rule() {
                    Rule::type_name => r#type = Some(self.parse_type_name(rule)?),
                    _ => unreachable!(),
                }
            }
            return Ok(Rc::new(RefCell::new(Expr {
                kind: ExprKind::SizeOf(r#type.unwrap()),
                ..Expr::new(span)
            })));
        }
        if rule.as_str().starts_with("alignof") {
            let span = rule.as_span();
            let mut r#type = None;
            for rule in rule.into_inner() {
                match rule.as_rule() {
                    Rule::type_name => r#type = Some(self.parse_type_name(rule)?),
                    _ => unreachable!(),
                }
            }
            return Ok(Rc::new(RefCell::new(Expr {
                kind: ExprKind::Alignof(r#type.unwrap()),
                ..Expr::new(span)
            })));
        }

        PRATT_PARSER
            .map_primary(|rule| self.parse_primary(rule))
            .map_prefix(|rule, operand| {
                let span = rule.as_span();
                Ok(Rc::new(RefCell::new(Expr {
                    kind: match rule.as_rule() {
                        Rule::cast => ExprKind::Cast {
                            is_implicit: false,
                            target: operand?,
                        },
                        _ => ExprKind::UnaryOp {
                            op: match rule.as_rule() {
                                Rule::positve => UnaryOpKind::Positive,
                                Rule::negative => UnaryOpKind::Negative,
                                Rule::bit_not => UnaryOpKind::BitNot,
                                Rule::not => UnaryOpKind::Not,
                                Rule::dereference => UnaryOpKind::Dereference,
                                Rule::addressof => UnaryOpKind::AddressOf,
                                Rule::sizeof => UnaryOpKind::SizeOf,
                                Rule::prefix_increment => UnaryOpKind::PrefixInc,
                                Rule::prefix_decrement => UnaryOpKind::PrefixDec,
                                _ => unreachable!(),
                            },
                            operand: operand?,
                        },
                    },
                    r#type: {
                        let span = rule.as_span();
                        let mut r#type = None;
                        for rule in rule.into_inner() {
                            match rule.as_rule() {
                                Rule::type_name => r#type = Some(self.parse_type_name(rule)?),
                                _ => {}
                            }
                        }
                        r#type.unwrap_or(Rc::new(RefCell::new(Type {
                            span,
                            attributes: vec![],
                            kind: TypeKind::Error,
                        })))
                    },
                    ..Expr::new(span)
                })))
            })
            .map_postfix(|operand, rule| {
                let span = rule.as_span();
                Ok(Rc::new(RefCell::new(Expr {
                    kind: match rule.as_rule() {
                        Rule::subscript => {
                            let mut expr = None;
                            for rule in rule.into_inner() {
                                match rule.as_rule() {
                                    Rule::expression => expr = Some(self.parse_expression(rule)?),
                                    _ => unreachable!(),
                                }
                            }
                            ExprKind::Subscript {
                                target: operand?,
                                index: expr.unwrap(),
                            }
                        }
                        Rule::function_call => {
                            let mut arguments = Vec::new();
                            for rule in rule.into_inner() {
                                match rule.as_rule() {
                                    Rule::assignment_expression => {
                                        arguments.push(self.parse_assignment_expression(rule)?)
                                    }
                                    _ => unreachable!(),
                                }
                            }
                            ExprKind::FunctionCall {
                                target: operand?,
                                arguments,
                            }
                        }
                        Rule::member_access => {
                            let is_arrow = rule.as_str().starts_with("->");
                            let mut name = String::new();
                            for rule in rule.into_inner() {
                                match rule.as_rule() {
                                    Rule::identifier => name = rule.as_str().to_string(),
                                    _ => unreachable!(),
                                }
                            }
                            ExprKind::MemberAccess {
                                target: operand?,
                                is_arrow,
                                name,
                            }
                        }
                        _ => ExprKind::UnaryOp {
                            op: match rule.as_rule() {
                                Rule::postfix_increment => UnaryOpKind::PostfixInc,
                                Rule::postfix_decrement => UnaryOpKind::PostfixDec,
                                _ => unreachable!(),
                            },
                            operand: operand?,
                        },
                    },
                    ..Expr::new(span)
                })))
            })
            .parse(rule.into_inner())
    }

    pub fn parse_logical_or_expression(
        &self,
        rule: Pair<'a, Rule>,
    ) -> Result<Rc<RefCell<Expr<'a>>>, Error<Rule>> {
        PRATT_PARSER
            .map_primary(|rule| match rule.as_rule() {
                Rule::unary_expression => self.parse_unary_expression(rule),
                _ => unreachable!(),
            })
            .map_infix(|left, rule, right| {
                Ok(Rc::new(RefCell::new(Expr {
                    kind: ExprKind::BinOp {
                        op: match rule.as_rule() {
                            Rule::add => BinOpKind::Add,
                            Rule::sub => BinOpKind::Sub,
                            Rule::mul => BinOpKind::Mul,
                            Rule::div => BinOpKind::Div,
                            Rule::r#mod => BinOpKind::Mod,
                            Rule::lshift => BinOpKind::LShift,
                            Rule::rshift => BinOpKind::RShift,
                            Rule::lt => BinOpKind::Lt,
                            Rule::le => BinOpKind::Le,
                            Rule::gt => BinOpKind::Gt,
                            Rule::ge => BinOpKind::Ge,
                            Rule::eq => BinOpKind::Eq,
                            Rule::neq => BinOpKind::Neq,
                            Rule::bit_and => BinOpKind::BitAnd,
                            Rule::bit_xor => BinOpKind::BitXOr,
                            Rule::bit_or => BinOpKind::BitOr,
                            Rule::and => BinOpKind::And,
                            Rule::or => BinOpKind::Or,
                            _ => unreachable!(),
                        },
                        left: left?,
                        right: right?,
                    },
                    ..Expr::new(rule.as_span())
                })))
            })
            .parse(rule.into_inner())
    }

    pub fn parse_primary(
        &self,
        rule: Pair<'a, Rule>,
    ) -> Result<Rc<RefCell<Expr<'a>>>, Error<Rule>> {
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::compound_literal => return self.parse_compound_literal(rule),
                Rule::generic_selection => return self.parse_generic_selection(rule),
                Rule::identifier => {
                    return Ok(Rc::new(RefCell::new(Expr {
                        kind: ExprKind::Name(rule.as_str().to_string()),
                        ..Expr::new(rule.as_span())
                    })));
                }
                Rule::constant => return self.parse_constant(rule),
                Rule::string_literal => return self.parse_string_literal(rule),
                Rule::expression => return self.parse_expression(rule),
                _ => unreachable!(),
            }
        }
        unreachable!()
    }

    pub fn parse_compound_literal(
        &self,
        rule: Pair<'a, Rule>,
    ) -> Result<Rc<RefCell<Expr<'a>>>, Error<Rule>> {
        let span = rule.as_span();
        let mut storage_classes = Vec::new();
        let mut r#type = None;
        let mut initializer = None;
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::storage_class_specifier => {
                    storage_classes.push(self.parse_storage_class_specifier(rule)?);
                }
                Rule::type_name => r#type = Some(self.parse_type_name(rule)?),
                Rule::braced_initializer => {
                    initializer = Some(self.parse_braced_initializer(rule)?)
                }
                _ => unreachable!(),
            }
        }
        Ok(Rc::new(RefCell::new(Expr {
            kind: ExprKind::CompoundLiteral {
                storage_classes,
                initializer: initializer.unwrap(),
            },
            r#type: r#type.unwrap(),
            ..Expr::new(span)
        })))
    }

    pub fn parse_generic_selection(
        &self,
        rule: Pair<'a, Rule>,
    ) -> Result<Rc<RefCell<Expr<'a>>>, Error<Rule>> {
        let span = rule.as_span();
        let mut expr = None;
        let mut assocs = Vec::new();
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::assignment_expression => expr = Some(self.parse_assignment_expression(rule)?),
                Rule::generic_association => assocs.push(self.parse_generic_association(rule)?),
                _ => unreachable!(),
            }
        }
        Ok(Rc::new(RefCell::new(Expr {
            kind: ExprKind::GenericSelection {
                control_expr: expr.unwrap(),
                assocs,
            },
            ..Expr::new(span)
        })))
    }

    pub fn parse_generic_association(
        &self,
        rule: Pair<'a, Rule>,
    ) -> Result<Rc<RefCell<GenericAssoc<'a>>>, Error<Rule>> {
        let span = rule.as_span();
        let mut r#type = None;
        let mut expr = None;

        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::type_name => r#type = Some(self.parse_type_name(rule)?),
                Rule::assignment_expression => expr = Some(self.parse_assignment_expression(rule)?),
                _ => unreachable!(),
            }
        }
        Ok(Rc::new(RefCell::new(GenericAssoc {
            span,
            is_selected: false,
            r#type,
            expr: expr.unwrap(),
        })))
    }

    pub fn parse_constant(
        &self,
        rule: Pair<'a, Rule>,
    ) -> Result<Rc<RefCell<Expr<'a>>>, Error<Rule>> {
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::floating_constant => return self.parse_floating_constant(rule),
                Rule::integer_constant => return self.parse_integer_constant(rule),
                Rule::character_constant => return self.parse_character_constant(rule),
                Rule::predefined_constant => return self.parse_predefined_constant(rule),
                _ => unreachable!(),
            }
        }
        unreachable!()
    }

    pub fn parse_floating_constant(
        &self,
        rule: Pair<'a, Rule>,
    ) -> Result<Rc<RefCell<Expr<'a>>>, Error<Rule>> {
        let mut base = 10;
        let mut digits = rule.as_str().to_string();
        let mut type_suffix = Vec::new();
        let mut exp_base = 10;
        let mut exponent = "0".to_string();
        if digits.starts_with("0x") || digits.starts_with("0X") {
            base = 16;
            digits = digits[2..].to_string();
        }
        for suffix in FLOAT_SUFFIX {
            while digits.ends_with(suffix) {
                type_suffix.push(suffix.to_string().to_lowercase());
                digits = digits[..digits.len() - suffix.len()].to_string();
            }
        }
        let mut digits = digits.to_lowercase();

        if digits.contains("p") {
            exp_base = 2;
            let i = digits.find("p").unwrap();
            exponent = digits[i + 1..].to_string();
            digits = digits[..i].to_string();
        } else if digits.contains("e") {
            let i = digits.find("e").unwrap();
            exponent = digits[i + 1..].to_string();
            digits = digits[..i].to_string();
        }

        Ok(Rc::new(RefCell::new(Expr {
            kind: ExprKind::Float {
                base,
                digits,
                exp_base,
                exponent,
                type_suffix,
            },
            ..Expr::new(rule.as_span())
        })))
    }

    pub fn parse_integer_constant(
        &self,
        rule: Pair<'a, Rule>,
    ) -> Result<Rc<RefCell<Expr<'a>>>, Error<Rule>> {
        let mut base = 10;
        let mut text = rule.as_str().to_string();
        let mut type_suffix = Vec::new();
        if text.starts_with("0x") || text.starts_with("0X") {
            base = 16;
            text = text[2..].to_string();
        } else if text.starts_with("0b") || text.starts_with("0B") {
            base = 2;
            text = text[2..].to_string();
        } else if text.starts_with("0") {
            base = 8;
            text = text[1..].to_string();
        }
        for suffix in INTEGER_SUFFIX {
            while text.ends_with(suffix) {
                type_suffix.push(suffix.to_string().to_lowercase());
                text = text[..text.len() - suffix.len()].to_string();
            }
        }

        Ok(Rc::new(RefCell::new(Expr {
            kind: ExprKind::Integer {
                base,
                text,
                type_suffix,
            },
            ..Expr::new(rule.as_span())
        })))
    }

    pub fn parse_predefined_constant(
        &self,
        rule: Pair<'a, Rule>,
    ) -> Result<Rc<RefCell<Expr<'a>>>, Error<Rule>> {
        Ok(Rc::new(RefCell::new(Expr {
            kind: match rule.as_str() {
                "true" => ExprKind::True,
                "false" => ExprKind::False,
                "nullptr" => ExprKind::Nullptr,
                _ => unreachable!(),
            },
            ..Expr::new(rule.as_span())
        })))
    }

    pub fn parse_character_constant(
        &self,
        rule: Pair<'a, Rule>,
    ) -> Result<Rc<RefCell<Expr<'a>>>, Error<Rule>> {
        let span = rule.as_span();
        let mut encode_prefix = "";
        let mut text = vec![];
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::encoding_prefix => encode_prefix = rule.as_str(),
                Rule::c_char => text.push(self.parse_char(rule)?),
                _ => unreachable!(),
            }
        }
        let text = text.iter().collect::<String>();
        Ok(Rc::new(RefCell::new(Expr {
            kind: ExprKind::Char {
                prefix: match encode_prefix {
                    "u8" => EncodePrefix::UTF8,
                    "u" => EncodePrefix::UTF16,
                    "U" => EncodePrefix::UTF32,
                    "L" => EncodePrefix::Wide,
                    _ => EncodePrefix::Default,
                },
                text,
            },
            ..Expr::new(span)
        })))
    }

    pub fn parse_char(&self, rule: Pair<'a, Rule>) -> Result<char, Error<Rule>> {
        let str = rule.as_str();
        for rule in rule.into_inner() {
            return Ok(match rule.as_rule() {
                Rule::escape_sequence => self.parse_escape_sequence(rule)?,
                _ => rule.as_str().chars().next().unwrap(),
            });
        }
        Ok(str.chars().next().unwrap())
    }

    pub fn parse_escape_sequence(&self, rule: Pair<'a, Rule>) -> Result<char, Error<Rule>> {
        for rule in rule.into_inner() {
            return Ok(match rule.as_rule() {
                Rule::simple_escape_sequence => match rule.as_str() {
                    "\\'" => '\'',
                    "\\\"" => '"',
                    "\\?" => unsafe { char::from_u32_unchecked(0x3f) }, // '\?'
                    "\\\\" => '\\',
                    "\\a" => unsafe { char::from_u32_unchecked(0x07) }, // '\a'
                    "\\b" => unsafe { char::from_u32_unchecked(0x08) }, // '\b'
                    "\\f" => unsafe { char::from_u32_unchecked(0x0c) }, // '\f'
                    "\\n" => '\n',
                    "\\r" => '\r',
                    "\\t" => '\t',
                    "\\v" => unsafe { char::from_u32_unchecked(0x0b) }, // '\v'
                    _ => unreachable!(),
                },
                Rule::octal_escape_sequence => unsafe {
                    char::from_u32_unchecked(u32::from_str_radix(&rule.as_str()[1..], 8).unwrap())
                },
                Rule::hexadecimal_escape_sequence | Rule::universal_character_name => unsafe {
                    char::from_u32_unchecked(u32::from_str_radix(&rule.as_str()[2..], 16).unwrap())
                },
                _ => rule.as_str().chars().next().unwrap(),
            });
        }
        Ok('\0')
    }

    pub fn parse_string_literal(
        &self,
        rule: Pair<'a, Rule>,
    ) -> Result<Rc<RefCell<Expr<'a>>>, Error<Rule>> {
        let span = rule.as_span();
        let mut encode_prefix = "";
        let mut text = vec![];
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::encoding_prefix => encode_prefix = rule.as_str(),
                Rule::s_char => text.push(self.parse_char(rule)?),
                _ => unreachable!(),
            }
        }
        let text = text.iter().collect::<String>();
        Ok(Rc::new(RefCell::new(Expr {
            span,
            kind: ExprKind::String {
                prefix: match encode_prefix {
                    "u8" => EncodePrefix::UTF8,
                    "u" => EncodePrefix::UTF16,
                    "U" => EncodePrefix::UTF32,
                    "L" => EncodePrefix::Wide,
                    _ => EncodePrefix::Default,
                },
                text,
            },
            ..Expr::new(span)
        })))
    }
}
