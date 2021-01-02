#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GenericValue {
    Void,
    Int1(bool),
    Int32(i32),
    Ptr(*mut u8),
}

impl GenericValue {
    pub fn to_ptr(&self) -> Option<*mut u8> {
        match self {
            Self::Ptr(p) => Some(*p),
            _ => None,
        }
    }

    pub fn to_i32(&self) -> Option<i32> {
        match self {
            Self::Int32(i) => Some(*i),
            _ => None,
        }
    }
}
