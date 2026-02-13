use num::{BigInt, BigRational, ToPrimitive, Zero};
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Rem, Shl, Shr, Sub};
use std::{fmt::Display, ops::Not, usize};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Variant {
    Int(BigInt),
    Array(Vec<Variant>),
    Bool(bool),
    Nullptr,
    Rational(BigRational),
    Unknown,
}

impl Variant {
    pub fn get(&self, index: &Variant) -> &Variant {
        match (self, index) {
            (Variant::Array(array), Variant::Int(index)) => array
                .get(index.to_usize().unwrap_or(usize::MAX))
                .unwrap_or(&Variant::Unknown),
            _ => &Variant::Unknown,
        }
    }

    ///返回None说明无法进行bool运算
    pub fn bool(&self) -> Option<bool> {
        match self {
            Variant::Int(value) => Some(*value != BigInt::ZERO),
            Variant::Array(array) => Some(array.len() > 0),
            Variant::Bool(value) => Some(*value),
            Variant::Nullptr => Some(false),
            Variant::Rational(value) => Some(*value.numer() != BigInt::ZERO),
            Variant::Unknown => None,
        }
    }

    pub fn and(&self, rhs: &Variant) -> Variant {
        match (self, rhs) {
            (Variant::Int(a), Variant::Int(b)) => {
                Variant::Int(BigInt::from(*a != BigInt::ZERO && *b != BigInt::ZERO))
            }
            (Variant::Rational(a), Variant::Rational(b)) => Variant::Int(BigInt::from(
                *a != BigRational::zero() && *b != BigRational::zero(),
            )),
            (Variant::Bool(a), Variant::Bool(b)) => Variant::Int(BigInt::from(*a && *b)),
            (Variant::Nullptr, Variant::Nullptr) => Variant::Int(BigInt::from(false)),
            _ => Variant::Unknown,
        }
    }

    pub fn or(&self, rhs: &Variant) -> Variant {
        match (self, rhs) {
            (Variant::Int(a), Variant::Int(b)) => {
                Variant::Int(BigInt::from(*a != BigInt::ZERO || *b != BigInt::ZERO))
            }
            (Variant::Rational(a), Variant::Rational(b)) => Variant::Int(BigInt::from(
                *a != BigRational::zero() || *b != BigRational::zero(),
            )),
            (Variant::Bool(a), Variant::Bool(b)) => Variant::Int(BigInt::from(*a || *b)),
            (Variant::Nullptr, Variant::Nullptr) => Variant::Int(BigInt::from(false)),
            _ => Variant::Unknown,
        }
    }
}

macro_rules! impl_ord {
    ($name:ident,$op:tt) => {
        impl Variant {
            pub fn $name(&self, rhs: &Variant) -> Variant {
                match (self, rhs) {
                    (Variant::Int(a), Variant::Int(b)) => Variant::Int(BigInt::from(*a $op *b)),
                    (Variant::Rational(a), Variant::Rational(b)) => {
                        Variant::Int(BigInt::from(*a $op *b))
                    }
                    (Variant::Bool(a), Variant::Bool(b)) => Variant::Int(BigInt::from(*a $op *b)),
                    (a @ Variant::Nullptr, b @ Variant::Nullptr) => Variant::Int(BigInt::from(*a $op *b)),
                    _ => Variant::Unknown,
                }
            }
        }
    };
}

impl_ord!(eq,==);
impl_ord!(neq,!=);
impl_ord!(lt,<);
impl_ord!(le,<=);
impl_ord!(gt,>);
impl_ord!(ge,>=);

impl Default for Variant {
    fn default() -> Self {
        Variant::Unknown
    }
}

impl Display for Variant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                Variant::Int(value) => format!("{value}"),
                Variant::Array(array) => format!(
                    "[{}]",
                    array
                        .iter()
                        .map(|x| format!("{x}"))
                        .collect::<Vec<String>>()
                        .join(", ")
                ),
                Variant::Bool(value) => format!("{value}"),
                Variant::Nullptr => format!("nullptr"),
                Variant::Rational(value) => format!("{value}"),
                Variant::Unknown => format!("unknown"),
            }
        )
    }
}

impl Not for Variant {
    type Output = Variant;

    fn not(self) -> Self::Output {
        match self {
            Variant::Int(value) => Variant::Int(!value),
            Variant::Bool(value) => Variant::Int(BigInt::from(!value)),
            Variant::Rational(value) => Variant::Int(BigInt::from(value == BigRational::zero())),
            Variant::Nullptr => Variant::Int(BigInt::from(true)),
            _ => Variant::Unknown,
        }
    }
}

impl Neg for Variant {
    type Output = Variant;

    fn neg(self) -> Self::Output {
        match self {
            Variant::Int(value) => Variant::Int(-value),
            Variant::Bool(value) => Variant::Int(-BigInt::from(value)),
            Variant::Rational(value) => Variant::Rational(-value),
            _ => Variant::Unknown,
        }
    }
}

impl Add for Variant {
    type Output = Variant;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Variant::Int(a), Variant::Int(b)) => Variant::Int(a + b),
            (Variant::Int(a), Variant::Rational(b)) | (Variant::Rational(b), Variant::Int(a)) => {
                Variant::Rational(BigRational::from_integer(a) + b)
            }
            (Variant::Bool(a), x) | (x, Variant::Bool(a)) => Variant::Int(BigInt::from(a)) + x,
            _ => Variant::Unknown,
        }
    }
}

impl Sub for Variant {
    type Output = Variant;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Variant::Int(a), Variant::Int(b)) => Variant::Int(a - b),
            (Variant::Int(a), Variant::Rational(b)) => {
                Variant::Rational(BigRational::from_integer(a) - b)
            }
            (Variant::Rational(b), Variant::Int(a)) => {
                Variant::Rational(b - BigRational::from_integer(a))
            }
            (Variant::Bool(a), x) => Variant::Int(BigInt::from(a)) - x,
            (x, Variant::Bool(a)) => x - Variant::Int(BigInt::from(a)),
            _ => Variant::Unknown,
        }
    }
}

impl Mul for Variant {
    type Output = Variant;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Variant::Int(a), Variant::Int(b)) => Variant::Int(a * b),
            (Variant::Int(a), Variant::Rational(b)) | (Variant::Rational(b), Variant::Int(a)) => {
                Variant::Rational(BigRational::from_integer(a) * b)
            }
            (Variant::Bool(a), x) | (x, Variant::Bool(a)) => Variant::Int(BigInt::from(a)) * x,
            _ => Variant::Unknown,
        }
    }
}

impl Div for Variant {
    type Output = Variant;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Variant::Int(a), Variant::Int(b)) => Variant::Int(a / b),
            (Variant::Int(a), Variant::Rational(b)) => {
                Variant::Rational(BigRational::from_integer(a) / b)
            }
            (Variant::Rational(b), Variant::Int(a)) => {
                Variant::Rational(b / BigRational::from_integer(a))
            }
            (Variant::Bool(a), x) => Variant::Int(BigInt::from(a)) / x,
            (x, Variant::Bool(a)) => x / Variant::Int(BigInt::from(a)),
            _ => Variant::Unknown,
        }
    }
}

impl Rem for Variant {
    type Output = Variant;

    fn rem(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Variant::Int(a), Variant::Int(b)) => Variant::Int(a % b),
            (Variant::Int(a), Variant::Rational(b)) => {
                Variant::Rational(BigRational::from_integer(a) % b)
            }
            (Variant::Rational(b), Variant::Int(a)) => {
                Variant::Rational(b % BigRational::from_integer(a))
            }
            (Variant::Bool(a), x) => Variant::Int(BigInt::from(a)) % x,
            (x, Variant::Bool(a)) => x % Variant::Int(BigInt::from(a)),
            _ => Variant::Unknown,
        }
    }
}

impl BitAnd for Variant {
    type Output = Variant;

    fn bitand(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Variant::Int(a), Variant::Int(b)) => Variant::Int(a & b),
            (Variant::Bool(a), x) | (x, Variant::Bool(a)) => Variant::Int(BigInt::from(a)) & x,
            _ => Variant::Unknown,
        }
    }
}

impl BitOr for Variant {
    type Output = Variant;

    fn bitor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Variant::Int(a), Variant::Int(b)) => Variant::Int(a | b),
            (Variant::Bool(a), x) | (x, Variant::Bool(a)) => Variant::Int(BigInt::from(a)) | x,
            _ => Variant::Unknown,
        }
    }
}

impl BitXor for Variant {
    type Output = Variant;

    fn bitxor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Variant::Int(a), Variant::Int(b)) => Variant::Int(a ^ b),
            (Variant::Bool(a), x) | (x, Variant::Bool(a)) => Variant::Int(BigInt::from(a)) ^ x,
            _ => Variant::Unknown,
        }
    }
}

impl Shl for Variant {
    type Output = Variant;

    fn shl(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            _ => Variant::Unknown,
        }
    }
}

impl Shr for Variant {
    type Output = Variant;

    fn shr(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            _ => Variant::Unknown,
        }
    }
}
