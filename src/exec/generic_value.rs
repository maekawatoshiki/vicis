use id_arena::Id;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GenericValue {
    Void,
    Int1(bool),
    Int32(i32),
    Ptr(*mut u8),
    Id([u8; 16]),
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

    pub fn to_id<T>(&self) -> Option<&T> {
        match self {
            Self::Id(id) => Some(unsafe { &*(id.as_ptr() as *const T) }),
            _ => None,
        }
    }

    pub fn id<T>(id: Id<T>) -> Self {
        Self::Id(unsafe { ::std::mem::transmute::<Id<T>, [u8; 16]>(id) })
    }
}
