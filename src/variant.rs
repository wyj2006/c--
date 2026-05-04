use bigdecimal::BigDecimal;
use num::{BigInt, BigRational, ToPrimitive};
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Rem, Shl, Shr, Sub};
use std::{fmt::Display, ops::Not, usize};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Variant {
    Int(BigInt),
    Array(Vec<Variant>),
    Bool(bool),
    Nullptr,
    Rational(BigRational),
    Complex(BigRational, BigRational),
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
            Variant::Complex(a, b) => {
                Some(*a.numer() != BigInt::ZERO && *b.numer() != BigInt::ZERO)
            }
            Variant::Unknown => None,
        }
    }

    pub fn and(&self, rhs: &Variant) -> Variant {
        match (self.bool(), rhs.bool()) {
            (Some(a), Some(b)) => Variant::Bool(a && b),
            _ => Variant::Unknown,
        }
    }

    pub fn or(&self, rhs: &Variant) -> Variant {
        match (self.bool(), rhs.bool()) {
            (Some(a), Some(b)) => Variant::Bool(a || b),
            _ => Variant::Unknown,
        }
    }

    pub fn is_unknown(&self) -> bool {
        match self {
            Variant::Unknown => true,
            Variant::Array(t) => t.iter().any(|x| x.is_unknown()),
            _ => false,
        }
    }
}

macro_rules! impl_ord {
    ($name:ident,$op:tt) => {
        impl Variant {
            pub fn $name(&self, rhs: &Variant) -> Variant {
                match (self, rhs) {
                    (Variant::Int(a), Variant::Int(b)) => Variant::Bool(*a $op *b),
                    (Variant::Rational(a), Variant::Rational(b)) => {
                        Variant::Bool(*a $op *b)
                    }
                    (Variant::Bool(a), x)|(x, Variant::Bool(a)) =>Variant::Int(BigInt::from(*a)).$name(x),
                    (a @ Variant::Nullptr, b @ Variant::Nullptr) => Variant::Bool(*a $op *b),
                    (Variant::Complex(a1,b1),Variant::Complex(a2,b2))=>Variant::Bool(*a1 $op *a2 && *b1 $op *b2),
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
                Variant::Complex(a, b) => format!("{a}+{b}i"),
                Variant::Unknown => format!("unknown"),
            }
        )
    }
}

#[macro_export]
macro_rules! forward_impl_binop {
    (impl $imp:ident for $res:ty, $method:ident) => {
        impl $imp<$res> for $res {
            type Output = $res;

            #[inline]
            fn $method(self, other: $res) -> $res {
                $imp::$method(&self, &other)
            }
        }

        impl $imp<$res> for &$res {
            type Output = $res;

            #[inline]
            fn $method(self, other: $res) -> $res {
                $imp::$method(self, &other)
            }
        }

        impl $imp<&$res> for $res {
            type Output = $res;

            #[inline]
            fn $method(self, other: &$res) -> $res {
                $imp::$method(&self, other)
            }
        }
    };
}

impl Not for &Variant {
    type Output = Variant;

    fn not(self) -> Self::Output {
        match self.bool() {
            Some(a) => Variant::Bool(!a),
            _ => Variant::Unknown,
        }
    }
}

impl Not for Variant {
    type Output = Variant;

    fn not(self) -> Self::Output {
        (&self).not()
    }
}

impl Neg for &Variant {
    type Output = Variant;

    fn neg(self) -> Self::Output {
        match self {
            Variant::Int(value) => Variant::Int(-value),
            Variant::Bool(value) => Variant::Int(-BigInt::from(*value)),
            Variant::Rational(value) => Variant::Rational(-value),
            Variant::Complex(a, b) => Variant::Complex(-a, -b),
            _ => Variant::Unknown,
        }
    }
}

impl Neg for Variant {
    type Output = Variant;

    fn neg(self) -> Self::Output {
        (&self).neg()
    }
}

impl Add<&Variant> for &Variant {
    type Output = Variant;

    fn add(self, rhs: &Variant) -> Self::Output {
        match (self, rhs) {
            (Variant::Int(a), Variant::Int(b)) => Variant::Int(a + b),
            (Variant::Rational(a), Variant::Rational(b)) => Variant::Rational(a + b),
            (Variant::Complex(a1, b1), Variant::Complex(a2, b2)) => {
                Variant::Complex(a1 + a2, b1 + b2)
            }
            (Variant::Int(a), Variant::Rational(b)) | (Variant::Rational(b), Variant::Int(a)) => {
                Variant::Rational(BigRational::from_integer(a.clone()) + b)
            }
            (Variant::Int(x), Variant::Complex(a, b))
            | (Variant::Complex(a, b), Variant::Int(x)) => Variant::Complex(a + x, b.clone()),
            (Variant::Rational(x), Variant::Complex(a, b))
            | (Variant::Complex(a, b), Variant::Rational(x)) => Variant::Complex(a + x, b.clone()),
            (Variant::Bool(a), x) | (x, Variant::Bool(a)) => Variant::Int(BigInt::from(*a)) + x,
            _ => Variant::Unknown,
        }
    }
}

forward_impl_binop!(impl Add for Variant,add);

impl Sub<&Variant> for &Variant {
    type Output = Variant;

    fn sub(self, rhs: &Variant) -> Self::Output {
        self + rhs.neg()
    }
}

forward_impl_binop!(impl Sub for Variant,sub);

impl Mul<&Variant> for &Variant {
    type Output = Variant;

    fn mul(self, rhs: &Variant) -> Self::Output {
        match (self, rhs) {
            (Variant::Int(a), Variant::Int(b)) => Variant::Int(a * b),
            (Variant::Rational(a), Variant::Rational(b)) => Variant::Rational(a * b),
            (Variant::Complex(a1, b1), Variant::Complex(a2, b2)) => {
                Variant::Complex(a1 * a2 - b1 * b2, a1 * b2 + a2 * b1)
            }
            (Variant::Int(a), Variant::Rational(b)) | (Variant::Rational(b), Variant::Int(a)) => {
                Variant::Rational(b * a)
            }
            (Variant::Int(x), Variant::Complex(a, b))
            | (Variant::Complex(a, b), Variant::Int(x)) => Variant::Complex(a * x, b * x),
            (Variant::Rational(x), Variant::Complex(a, b))
            | (Variant::Complex(a, b), Variant::Rational(x)) => Variant::Complex(a * x, b * x),
            (Variant::Bool(a), x) | (x, Variant::Bool(a)) => Variant::Int(BigInt::from(*a)) * x,
            _ => Variant::Unknown,
        }
    }
}

forward_impl_binop!(impl Mul for Variant,mul);

impl Div<&Variant> for &Variant {
    type Output = Variant;

    fn div(self, rhs: &Variant) -> Self::Output {
        match (self, rhs) {
            (Variant::Int(a), Variant::Int(b)) => Variant::Int(a / b),
            (Variant::Rational(a), Variant::Rational(b)) => Variant::Rational(a / b),
            (Variant::Complex(a1, b1), Variant::Complex(a2, b2)) => {
                /*
                (a1+b1*i)/(a2+b2*i)
                =(a1+b1*i)*(a2-b2*i)/(a2*a2+b2*b2)
                =[a1*a2+b1*b2+(b1*a2-a1*b2)*i]/(a2*a2+b2*b2)
                 */
                Variant::Complex(
                    (a1 * a2 + b1 * b2) / (a2 * a2 + b2 * b2),
                    (b1 * a2 - a1 * b2) / (a2 * a2 + b2 * b2),
                )
            }
            (Variant::Int(a), Variant::Rational(b)) => {
                Variant::Rational(BigRational::from_integer(a.clone()) / b)
            }
            (Variant::Rational(b), Variant::Int(a)) => Variant::Rational(b / a),
            (Variant::Complex(a, b), Variant::Int(x)) => Variant::Complex(a / x, b / x),
            //x/(a+b*i)=x*(a-b*i)/(a*a+b*b)=(x*a-x*b*i)/(a*a+b*b)
            (Variant::Int(x), Variant::Complex(a, b)) => {
                Variant::Complex(a * x / (a * a + b * b), -b * x / (a * a + b * b))
            }
            (Variant::Complex(a, b), Variant::Rational(x)) => Variant::Complex(a / x, b / x),
            (Variant::Rational(x), Variant::Complex(a, b)) => {
                Variant::Complex(a * x / (a * a + b * b), -b * x / (a * a + b * b))
            }
            (Variant::Bool(a), x) => Variant::Int(BigInt::from(*a)) / x,
            (x, Variant::Bool(a)) => x / Variant::Int(BigInt::from(*a)),
            _ => Variant::Unknown,
        }
    }
}

forward_impl_binop!(impl Div for Variant,div);

impl Rem<&Variant> for &Variant {
    type Output = Variant;

    fn rem(self, rhs: &Variant) -> Self::Output {
        match (self, rhs) {
            (Variant::Int(a), Variant::Int(b)) => Variant::Int(a % b),
            (Variant::Rational(a), Variant::Rational(b)) => Variant::Rational(a % b),
            (Variant::Int(a), Variant::Rational(b)) => {
                Variant::Rational(BigRational::from_integer(a.clone()) % b)
            }
            (Variant::Rational(b), Variant::Int(a)) => {
                Variant::Rational(b % BigRational::from_integer(a.clone()))
            }
            (Variant::Bool(a), x) => Variant::Int(BigInt::from(*a)) % x,
            (x, Variant::Bool(a)) => x % Variant::Int(BigInt::from(*a)),
            _ => Variant::Unknown,
        }
    }
}

forward_impl_binop!(impl Rem for Variant,rem);

impl BitAnd<&Variant> for &Variant {
    type Output = Variant;

    fn bitand(self, rhs: &Variant) -> Self::Output {
        match (self, rhs) {
            (Variant::Int(a), Variant::Int(b)) => Variant::Int(a & b),
            (Variant::Bool(a), x) | (x, Variant::Bool(a)) => Variant::Int(BigInt::from(*a)) & x,
            _ => Variant::Unknown,
        }
    }
}

forward_impl_binop!(impl BitAnd for Variant,bitand);

impl BitOr<&Variant> for &Variant {
    type Output = Variant;

    fn bitor(self, rhs: &Variant) -> Self::Output {
        match (self, rhs) {
            (Variant::Int(a), Variant::Int(b)) => Variant::Int(a | b),
            (Variant::Bool(a), x) | (x, Variant::Bool(a)) => Variant::Int(BigInt::from(*a)) | x,
            _ => Variant::Unknown,
        }
    }
}

forward_impl_binop!(impl BitOr for Variant,bitor);

impl BitXor<&Variant> for &Variant {
    type Output = Variant;

    fn bitxor(self, rhs: &Variant) -> Self::Output {
        match (self, rhs) {
            (Variant::Int(a), Variant::Int(b)) => Variant::Int(a ^ b),
            (Variant::Bool(a), x) | (x, Variant::Bool(a)) => Variant::Int(BigInt::from(*a)) ^ x,
            _ => Variant::Unknown,
        }
    }
}

forward_impl_binop!(impl BitXor for Variant,bitxor);

impl Shl<&Variant> for &Variant {
    type Output = Variant;

    fn shl(self, rhs: &Variant) -> Self::Output {
        match (self, rhs) {
            (Variant::Int(a), Variant::Int(b)) => {
                if let Some(t) = b.to_isize() {
                    Variant::Int(a << t)
                } else if let Some(t) = b.to_usize() {
                    Variant::Int(a << t)
                } else {
                    Variant::Unknown
                }
            }
            (Variant::Bool(a), _) if *a == false => Variant::Int(BigInt::ZERO),
            (Variant::Bool(a), x) => Variant::Int(BigInt::from(*a)).shl(x),
            (x, Variant::Bool(a)) => x.shl(Variant::Int(BigInt::from(*a))),
            _ => Variant::Unknown,
        }
    }
}

forward_impl_binop!(impl Shl for Variant,shl);

impl Shr<&Variant> for &Variant {
    type Output = Variant;

    fn shr(self, rhs: &Variant) -> Self::Output {
        match (self, rhs) {
            (Variant::Int(a), Variant::Int(b)) => {
                if let Some(t) = b.to_isize() {
                    Variant::Int(a >> t)
                } else if let Some(t) = b.to_usize() {
                    Variant::Int(a >> t)
                } else {
                    Variant::Unknown
                }
            }
            (Variant::Bool(a), _) if *a == false => Variant::Int(BigInt::ZERO),
            (Variant::Bool(a), x) => Variant::Int(BigInt::from(*a)).shr(x),
            (x, Variant::Bool(a)) => x.shr(Variant::Int(BigInt::from(*a))),
            _ => Variant::Unknown,
        }
    }
}

forward_impl_binop!(impl Shr for Variant,shr);

pub fn to_decimal(a: &BigRational) -> BigDecimal {
    BigDecimal::from(a.numer().clone()) / BigDecimal::from(a.denom().clone())
}
