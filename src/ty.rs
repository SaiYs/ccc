#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    I64,
    Ptr {
        to: Box<Type>,
    },
    Array {
        element: Box<Type>,
        len: usize,
    },
    Fn {
        args: Vec<Type>,
        ret: Box<Type>,
    },
    Void,
    Never,

    #[allow(dead_code)]
    Unknown,
}

impl Type {
    pub fn size(&self) -> usize {
        match self {
            Type::I64 => 8,
            Type::Ptr { .. } => 8,
            Type::Array { element, len } => element.size() * len,
            _ => todo!(),
        }
    }
}
