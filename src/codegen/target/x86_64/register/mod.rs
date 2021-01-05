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
        Reg(self as u32)
    }
}

impl Into<Reg> for GR64 {
    fn into(self) -> Reg {
        Reg(self as u32 + 16 /*=num of GR32 regs*/)
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
