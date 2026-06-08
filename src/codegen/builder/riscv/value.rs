use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Clone)]
pub enum RISCVValue {
    Integer(u64),
    Float(f32),
    Double(f64),
    Array(Vec<Rc<RefCell<RISCVValue>>>),
    Struct(Vec<Rc<RefCell<RISCVValue>>>),
    //整数寄存器, 当索引大于31时为虚拟寄存器
    IReg(usize),
    //浮点数寄存器,  当索引大于31时为虚拟寄存器
    FReg(usize),
    //寻址
    Address(Rc<RefCell<RISCVValue>>, isize),
    Function { name: String, frame_size: usize },
    Symbol(String),
}

impl RISCVValue {
    pub fn new_complex(
        real: Rc<RefCell<RISCVValue>>,
        imag: Rc<RefCell<RISCVValue>>,
    ) -> Rc<RefCell<RISCVValue>> {
        Rc::new(RefCell::new(RISCVValue::Struct(vec![real, imag])))
    }

    pub fn is_const(&self) -> bool {
        match self {
            RISCVValue::Integer(_) | RISCVValue::Float(_) | RISCVValue::Double(_) => true,
            RISCVValue::Array(t) | RISCVValue::Struct(t) => t.iter().all(|x| x.borrow().is_const()),
            _ => false,
        }
    }
}
