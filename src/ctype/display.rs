use crate::ctype::RecordKind;

use super::{Type, TypeKind};
use std::fmt::Display;

#[derive(Debug, Default)]
struct TypeString {
    pub prefix: Vec<String>,
    pub infix: Vec<String>,
    pub child: Option<Box<TypeString>>,
    pub postfix: Vec<String>,
}

impl Display for TypeString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.prefix.len() > 0 {
            write!(f, "{}", self.prefix.join(" "))?;
        }
        write!(f, "{}", self.infix.join(""))?;
        if let Some(t) = &self.child {
            write!(f, "({})", t)?;
        }
        write!(f, "{}", self.postfix.join(""))
    }
}

impl Type<'_> {
    fn to_typestring(&self, mut parent: TypeString) -> TypeString {
        match &self.kind {
            TypeKind::Void => parent.prefix.push("void".to_string()),
            TypeKind::Bool => parent.prefix.push("void".to_string()),
            TypeKind::Char => parent.prefix.push("char".to_string()),
            TypeKind::SignedChar => parent.prefix.push("signed char".to_string()),
            TypeKind::UnsignedChar => parent.prefix.push("unsigned char".to_string()),
            TypeKind::Signed => parent.prefix.push("signed".to_string()),
            TypeKind::Unsigned => parent.prefix.push("unsigned".to_string()),
            TypeKind::Short => parent.prefix.push("short".to_string()),
            TypeKind::UShort => parent.prefix.push("unsigned short".to_string()),
            TypeKind::Int => parent.prefix.push("int".to_string()),
            TypeKind::UInt => parent.prefix.push("unsigned int".to_string()),
            TypeKind::Long => parent.prefix.push("long".to_string()),
            TypeKind::ULong => parent.prefix.push("unsigned long".to_string()),
            TypeKind::LongLong => parent.prefix.push("long long".to_string()),
            TypeKind::ULongLong => parent.prefix.push("unsigned long long".to_string()),
            TypeKind::BitInt {
                unsigned,
                width_expr: _,
            } => {
                parent.prefix.push(format!(
                    "{}_BitInt()",
                    if *unsigned { "unsigned " } else { "" }
                ));
            }
            TypeKind::Float => parent.prefix.push("float".to_string()),
            TypeKind::Double => parent.prefix.push("double".to_string()),
            TypeKind::LongDouble => parent.prefix.push("long double".to_string()),
            TypeKind::Decimal32 => parent.prefix.push("_Decimal32".to_string()),
            TypeKind::Decimal64 => parent.prefix.push("_Decimal64".to_string()),
            TypeKind::Decimal128 => parent.prefix.push("_Decimal128".to_string()),
            TypeKind::Complex | TypeKind::FloatComplex => {
                parent.prefix.push("float _Complex".to_string())
            }
            TypeKind::DoubleComplex => parent.prefix.push("double _Complex".to_string()),
            TypeKind::LongDoubleComplex => parent.prefix.push("long double _Complex".to_string()),
            TypeKind::Function {
                return_type,
                parameters_type,
                has_varparam,
            } => {
                let mut parameters_string = parameters_type
                    .iter()
                    .map(|x| x.borrow().to_string())
                    .collect::<Vec<String>>();
                if *has_varparam {
                    parameters_string.push("...".to_string());
                }
                parent
                    .postfix
                    .insert(0, format!("({})", parameters_string.join(",")));
                parent = return_type.borrow().to_typestring(parent);
            }
            TypeKind::Qualified { qualifiers, r#type } => {
                if !r#type.borrow().is_array() && !r#type.borrow().is_pointer() {
                    parent.prefix.extend(
                        qualifiers
                            .iter()
                            .map(|x| x.to_string())
                            .collect::<Vec<String>>(),
                    );
                }
            }
            TypeKind::Typeof {
                unqual,
                expr,
                r#type,
            } => {
                parent.prefix.push(format!(
                    "typeof{}({}{})",
                    if *unqual { "_unqual" } else { "" },
                    if let Some(_) = expr {
                        //TODO
                        todo!()
                    } else {
                        "".to_string()
                    },
                    if let Some(t) = r#type {
                        format!("{}", t.borrow())
                    } else {
                        "".to_string()
                    }
                ));
            }
            TypeKind::Typedef { name, r#type } => {
                parent.prefix.push(format!(
                    "{name}:{}",
                    if let Some(t) = r#type {
                        format!("{}", t.borrow())
                    } else {
                        "<unkown>".to_string()
                    }
                ));
            }
            TypeKind::Record { name, kind } => {
                parent.prefix.push(format!(
                    "{} {name}",
                    match kind {
                        RecordKind::Struct => "struct",
                        RecordKind::Union => "union",
                    }
                ));
            }
            TypeKind::Enum { name, underlying } => {
                parent
                    .prefix
                    .push(format!("enum {name}:{}", underlying.borrow()));
            }
            TypeKind::Atomic(t) => {
                parent.prefix.push(format!("_Atomic({})", t.borrow()));
            }
            _ => {}
        }

        if self.is_pointer() {
            let mut quals = &Vec::new();
            let insider;
            let pointee;
            if let TypeKind::Qualified { qualifiers, r#type } = &self.kind {
                quals = qualifiers;
                insider = r#type.borrow();
                if let TypeKind::Pointer(t) = &insider.kind {
                    pointee = t.borrow();
                } else {
                    unreachable!()
                }
            } else if let TypeKind::Pointer(t) = &self.kind {
                pointee = t.borrow();
            } else {
                unreachable!();
            }

            parent.infix.insert(
                0,
                format!(
                    "*{}",
                    quals
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>()
                        .join(" "),
                ),
            );
            if pointee.is_array() || pointee.is_function() {
                parent = TypeString {
                    child: Some(Box::new(parent)),
                    ..pointee.to_typestring(TypeString::default())
                };
            } else {
                parent = pointee.to_typestring(parent);
            }
        }

        if self.is_array() {
            let mut quals = &Vec::new();
            let insider;
            let has_static;
            let has_star;
            let element_type;
            let len_expr;
            if let TypeKind::Qualified { qualifiers, r#type } = &self.kind {
                quals = qualifiers;
                insider = r#type.borrow();
                if let TypeKind::Array {
                    has_static: a,
                    has_star: b,
                    element_type: c,
                    len_expr: d,
                } = &insider.kind
                {
                    has_static = *a;
                    has_star = *b;
                    element_type = c;
                    len_expr = d;
                } else {
                    unreachable!()
                };
            } else if let TypeKind::Array {
                has_static: a,
                has_star: b,
                element_type: c,
                len_expr: d,
            } = &self.kind
            {
                has_static = *a;
                has_star = *b;
                element_type = c;
                len_expr = d;
            } else {
                unreachable!()
            };

            parent.postfix.insert(
                0,
                format!(
                    "[{}{}{}{}]",
                    if has_static { "static" } else { "" },
                    quals
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>()
                        .join(" "),
                    if has_star { "*" } else { "" },
                    if let Some(_) = len_expr {
                        //TODO
                        todo!()
                    } else {
                        "".to_string()
                    }
                ),
            );

            parent = element_type.borrow().to_typestring(parent);
        }

        parent
    }
}

impl Display for Type<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_typestring(TypeString::default()))
    }
}
