#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GenericValue {
    Void,
    Int32(i32),
    Ptr(*mut u8),
}

impl GenericValue {
    pub fn as_ptr(&self) -> Option<*mut u8> {
        match self {
            Self::Ptr(p) => Some(*p),
            _ => None,
        }
    }
}
