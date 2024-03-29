use id_arena::Id;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GenericValue {
    Void,
    Int1(bool),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Ptr(*mut u8),
    Id([u8; 16]),
}

impl GenericValue {
    pub const fn to_ptr(self) -> Option<*mut u8> {
        match self {
            Self::Ptr(p) => Some(p),
            _ => None,
        }
    }

    pub const fn to_i1(self) -> Option<bool> {
        match self {
            Self::Int1(i) => Some(i),
            _ => None,
        }
    }

    pub const fn to_i8(self) -> Option<i8> {
        match self {
            Self::Int8(i) => Some(i),
            _ => None,
        }
    }

    pub const fn to_i32(self) -> Option<i32> {
        match self {
            Self::Int32(i) => Some(i),
            _ => None,
        }
    }

    pub const fn to_i64(self) -> Option<i64> {
        match self {
            Self::Int64(i) => Some(i),
            _ => None,
        }
    }

    pub const fn sext_to_i64(&self) -> Option<i64> {
        match self {
            Self::Int1(i) => Some(*i as i64),
            Self::Int8(i) => Some(*i as i64),
            Self::Int16(i) => Some(*i as i64),
            Self::Int32(i) => Some(*i as i64),
            Self::Int64(i) => Some(*i),
            _ => None,
        }
    }

    pub const fn zext_to_u64(&self) -> Option<u64> {
        match self {
            Self::Int1(i) => Some(*i as u64),
            Self::Int8(i) => Some(*i as u64),
            Self::Int16(i) => Some(*i as u64),
            Self::Int32(i) => Some(*i as u64),
            Self::Int64(i) => Some(*i as u64),
            _ => None,
        }
    }

    pub const fn to_id<T>(&self) -> Option<&T> {
        match self {
            Self::Id(id) => Some(unsafe { &*(id.as_ptr() as *const T) }),
            _ => None,
        }
    }

    pub const fn id<T>(id: Id<T>) -> Self {
        Self::Id(unsafe { ::std::mem::transmute::<Id<T>, [u8; 16]>(id) })
    }
}
