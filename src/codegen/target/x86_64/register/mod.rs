use crate::{
    codegen::{
        call_conv::CallConvKind,
        register::{Reg, RegUnit, RegisterClass},
    },
    ir::types::{Type, TypeId, Types},
};
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

pub enum RegClass {
    GR32,
    GR64,
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

impl RegisterClass for RegClass {
    fn for_type(types: &Types, id: TypeId) -> Self {
        match &*types.get(id) {
            Type::Int(32) => Self::GR32,
            Type::Int(64) => Self::GR64,
            _ => todo!(),
        }
    }

    fn gpr_list_for(rc: &Self) -> Vec<Reg> {
        match rc {
            // TODO: Add more general-purpose registers
            RegClass::GR32 => vec![GR32::EAX, GR32::ECX, GR32::EDX]
                .into_iter()
                .map(|r| r.into())
                .collect(),
            // TODO: Add more general-purpose registers
            RegClass::GR64 => vec![GR64::RAX, GR64::RCX, GR64::RDX]
                .into_iter()
                .map(|r| r.into())
                .collect(),
        }
    }

    fn arg_reg_list_for(_rc: &Self, _cc: &CallConvKind) -> Vec<Reg> {
        todo!()
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
