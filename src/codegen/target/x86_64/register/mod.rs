use crate::codegen::register::Reg;
use std::fmt;

pub enum GR32 {
    EAX,
}

pub enum GR64 {
    RBP,
    RSP,
}

impl Into<Reg> for GR32 {
    fn into(self) -> Reg {
        Reg(0, self as u16)
    }
}

impl Into<Reg> for GR64 {
    fn into(self) -> Reg {
        Reg(1, self as u16)
    }
}

impl fmt::Debug for GR64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::RBP => "rbp",
                Self::RSP => "rsp",
            }
        )
    }
}

impl fmt::Display for GR64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Debug for GR32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::EAX => "eax",
            }
        )
    }
}

impl fmt::Display for GR32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
