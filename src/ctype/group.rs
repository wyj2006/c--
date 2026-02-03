use crate::ctype::{RecordKind, Type, TypeKind};

impl Type {
    pub fn is_pointer(&self) -> bool {
        self.kind.is_pointer()
    }

    pub fn is_array(&self) -> bool {
        self.kind.is_array()
    }

    pub fn is_function(&self) -> bool {
        self.kind.is_function()
    }

    pub fn is_real_float(&self) -> bool {
        self.kind.is_real_float()
    }

    pub fn is_complex(&self) -> bool {
        self.kind.is_complex()
    }

    pub fn is_void(&self) -> bool {
        self.kind.is_void()
    }

    pub fn is_void_ptr(&self) -> bool {
        self.kind.is_void_ptr()
    }

    pub fn is_float(&self) -> bool {
        self.kind.is_float()
    }

    pub fn is_arithmetic(&self) -> bool {
        self.kind.is_arithmetic()
    }

    pub fn is_scale(&self) -> bool {
        self.kind.is_scale()
    }

    pub fn is_real(&self) -> bool {
        self.kind.is_real()
    }

    pub fn is_nullptr(&self) -> bool {
        self.kind.is_nullptr()
    }

    pub fn is_union(&self) -> bool {
        self.kind.is_union()
    }

    pub fn is_object(&self) -> bool {
        self.kind.is_object()
    }

    pub fn is_char(&self) -> bool {
        self.kind.is_char()
    }
}

impl TypeKind {
    pub fn is_pointer(&self) -> bool {
        match self {
            TypeKind::Pointer(_) => true,
            TypeKind::Qualified { r#type, .. } => r#type.borrow().is_pointer(),
            _ => false,
        }
    }

    pub fn is_array(&self) -> bool {
        match self {
            TypeKind::Array { .. } => true,
            TypeKind::Qualified { r#type, .. } => r#type.borrow().is_array(),
            _ => false,
        }
    }

    pub fn is_function(&self) -> bool {
        match self {
            TypeKind::Function { .. } => true,
            TypeKind::Qualified { r#type, .. } => r#type.borrow().is_function(),
            _ => false,
        }
    }

    pub fn is_real_float(&self) -> bool {
        match self {
            TypeKind::Float | TypeKind::Double | TypeKind::LongDouble => true,
            TypeKind::Qualified { r#type, .. } => r#type.borrow().is_real_float(),
            _ => false,
        }
    }

    pub fn is_complex(&self) -> bool {
        match self {
            TypeKind::Complex(..) => true,
            TypeKind::Qualified { r#type, .. } => r#type.borrow().is_complex(),
            _ => false,
        }
    }

    pub fn is_void(&self) -> bool {
        match self {
            TypeKind::Void => true,
            _ => false,
        }
    }

    pub fn is_void_ptr(&self) -> bool {
        match self {
            TypeKind::Pointer(pointee) => pointee.borrow().is_void(),
            TypeKind::Qualified { r#type, .. } => r#type.borrow().is_void_ptr(),
            _ => false,
        }
    }

    pub fn is_float(&self) -> bool {
        match self {
            TypeKind::Decimal32
            | TypeKind::Decimal64
            | TypeKind::Decimal128
            | TypeKind::Complex(..) => true,
            TypeKind::Qualified { r#type, .. } => r#type.borrow().is_float(),
            _ => self.is_real_float(),
        }
    }

    pub fn is_arithmetic(&self) -> bool {
        self.is_integer() || self.is_float()
    }

    pub fn is_scale(&self) -> bool {
        match self {
            TypeKind::Nullptr => true,
            _ => self.is_arithmetic() || self.is_pointer(),
        }
    }

    pub fn is_real(&self) -> bool {
        self.is_integer() || self.is_real_float()
    }

    pub fn is_nullptr(&self) -> bool {
        match self {
            TypeKind::Nullptr => true,
            _ => false,
        }
    }

    pub fn is_union(&self) -> bool {
        match self {
            TypeKind::Record {
                kind: RecordKind::Union,
                ..
            } => true,
            TypeKind::Qualified { r#type, .. } => r#type.borrow().is_union(),
            _ => false,
        }
    }

    pub fn is_object(&self) -> bool {
        match self {
            TypeKind::Function { .. } => false,
            _ => true,
        }
    }

    pub fn is_char(&self) -> bool {
        match self {
            TypeKind::Char | TypeKind::UnsignedChar | TypeKind::SignedChar => true,
            _ => false,
        }
    }
}
