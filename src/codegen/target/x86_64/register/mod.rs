use crate::codegen::register::{Reg, RegUnit};
use std::fmt;

pub enum GR32 {
    EAX,
    ECX,
    EDX,
    EBX,
    ESP,
    EBP,
    ESI,
    EDI,
    R8D,
    R9D,
    R10D,
    R11D,
    R12D,
    R13D,
    R14D,
    R15D,
}

pub enum GR64 {
    RAX,
    RCX,
    RDX,
    RBX,
    RSP,
    RBP,
    RSI,
    RDI,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
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

pub fn to_reg_unit(r: Reg) -> RegUnit {
    match r {
        Reg(/*GR32*/ 0, x) => RegUnit(0, x),
        Reg(/*GR64*/ 1, x) => RegUnit(0, x),
        _ => todo!(),
    }
}

impl fmt::Debug for GR64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::RAX => "rax",
                Self::RCX => "rcx",
                Self::RDX => "rdx",
                Self::RBX => "rbx",
                Self::RSP => "rsp",
                Self::RBP => "rbp",
                Self::RSI => "rsi",
                Self::RDI => "rdi",
                Self::R8 => "r8",
                Self::R9 => "r9",
                Self::R10 => "r10",
                Self::R11 => "r11",
                Self::R12 => "r12",
                Self::R13 => "r13",
                Self::R14 => "r14",
                Self::R15 => "r15",
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
                Self::ECX => "ecx",
                Self::EDX => "edx",
                Self::EBX => "ebx",
                Self::ESP => "esp",
                Self::EBP => "ebp",
                Self::ESI => "esi",
                Self::EDI => "edi",
                Self::R8D => "r8",
                Self::R9D => "r9d",
                Self::R10D => "r10d",
                Self::R11D => "r11d",
                Self::R12D => "r12d",
                Self::R13D => "r13d",
                Self::R14D => "r14d",
                Self::R15D => "r15d",
            }
        )
    }
}

impl fmt::Display for GR32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
