use crate::{
    call_conv::CallConvKind,
    register::{Reg, RegUnit, RegisterClass, RegisterInfo},
};
use std::fmt;
use vicis_core::ir::types::{self, Type, Types};

pub struct RegInfo;

pub enum GR8 {
    // TODO: AH, CH, DH, BH
    AL,
    CL,
    DL,
    BL,
    SIL,
    DIL,
    BPL,
    SPL,
    R8B,
    R9B,
    R10B,
    R11B,
    R12B,
    R13B,
    R14B,
    R15B,
    IP,
}

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
    EIP,
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
    RIP,
}

pub enum RegClass {
    GR8,
    GR32,
    GR64,
}

impl From<GR8> for Reg {
    fn from(r: GR8) -> Self {
        Reg(RegClass::GR8 as u16, r as u16)
    }
}

impl From<GR32> for Reg {
    fn from(r: GR32) -> Self {
        Reg(RegClass::GR32 as u16, r as u16)
    }
}

impl From<GR64> for Reg {
    fn from(r: GR64) -> Self {
        Reg(RegClass::GR64 as u16, r as u16)
    }
}

impl From<GR8> for RegUnit {
    fn from(r: GR8) -> Self {
        RegUnit(RegClass::GR64 as u16, r as u16)
    }
}

impl From<GR32> for RegUnit {
    fn from(r: GR32) -> Self {
        RegUnit(RegClass::GR64 as u16, r as u16)
    }
}

impl From<GR64> for RegUnit {
    fn from(r: GR64) -> Self {
        RegUnit(RegClass::GR64 as u16, r as u16)
    }
}

const ARG_REGS: [RegUnit; 6] = [
    RegUnit(RegClass::GR64 as u16, GR64::RDI as u16),
    RegUnit(RegClass::GR64 as u16, GR64::RSI as u16),
    RegUnit(RegClass::GR64 as u16, GR64::RDX as u16),
    RegUnit(RegClass::GR64 as u16, GR64::RCX as u16),
    RegUnit(RegClass::GR64 as u16, GR64::R8 as u16),
    RegUnit(RegClass::GR64 as u16, GR64::R9 as u16),
];

impl RegisterInfo for RegInfo {
    fn arg_reg_list(cc: &CallConvKind) -> &'static [RegUnit] {
        match cc {
            CallConvKind::SystemV => &ARG_REGS,
            CallConvKind::AAPCS64 => &[],
        }
    }

    fn to_reg_unit(r: Reg) -> RegUnit {
        match r {
            Reg(/*GR8*/ 0, x) => RegUnit(RegClass::GR64 as u16, x),
            Reg(/*GR32*/ 1, x) => RegUnit(RegClass::GR64 as u16, x),
            Reg(/*GR64*/ 2, x) => RegUnit(RegClass::GR64 as u16, x),
            _ => panic!(),
        }
    }
}

impl RegisterClass for RegClass {
    fn for_type(types: &Types, ty: Type) -> Self {
        match ty {
            types::I8 => RegClass::GR8,
            types::I32 => RegClass::GR32,
            types::I64 => RegClass::GR64,
            _ if ty.is_pointer(types) => RegClass::GR64,
            e => todo!("{}", types.to_string(e)),
        }
    }

    fn gpr_list(&self) -> Vec<Reg> {
        match self {
            // TODO: Add more general-purpose registers
            RegClass::GR8 => vec![GR8::AL, GR8::CL, GR8::DL, GR8::DIL, GR8::SIL]
                .into_iter()
                .map(Into::into)
                .collect(),
            RegClass::GR32 => vec![GR32::EAX, GR32::ECX, GR32::EDX, GR32::EDI, GR32::ESI]
                .into_iter()
                .map(|r| r.into())
                .collect(),
            // TODO: Add more general-purpose registers
            RegClass::GR64 => vec![GR64::RAX, GR64::RCX, GR64::RDX, GR64::RDI, GR64::RSI]
                .into_iter()
                .map(|r| r.into())
                .collect(),
        }
    }

    fn apply_for(&self, ru: RegUnit) -> Reg {
        match self {
            Self::GR8 => Reg(RegClass::GR8 as u16, ru.1),
            Self::GR32 => Reg(RegClass::GR32 as u16, ru.1),
            Self::GR64 => Reg(RegClass::GR64 as u16, ru.1),
        }
    }
}

pub fn to_reg_unit(r: Reg) -> RegUnit {
    match r {
        Reg(/*GR8*/ 0, x) => RegUnit(2, x),
        Reg(/*GR32*/ 1, x) => RegUnit(2, x),
        Reg(/*GR64*/ 2, x) => RegUnit(2, x),
        _ => panic!(),
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
                Self::RIP => "rip",
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
                Self::EIP => "eip",
            }
        )
    }
}

impl fmt::Display for GR32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn reg_to_str(r: &Reg) -> &'static str {
    let gr8 = [
        "al", "cl", "dl", "ah", "ch", "dh", "bl", "bh", "sil", "dil", "bpl", "spl", "r8b", "r9b",
        "r10b", "r11b", "r12b", "r13b", "r14b", "r15b", "ip",
    ];
    let gr32 = [
        "eax", "ecx", "edx", "ebx", "esp", "ebp", "esi", "edi", "r8", "r9d", "r10d", "r11d",
        "r12d", "r13d", "r14d", "r15d", "eip",
    ];
    let gr64 = [
        "rax", "rcx", "rdx", "rbx", "rsp", "rbp", "rsi", "rdi", "r8", "r9", "r10", "r11", "r12",
        "r13", "r14", "r15", "rip",
    ];
    match r {
        Reg(0, i) => gr8[*i as usize],
        Reg(1, i) => gr32[*i as usize],
        Reg(2, i) => gr64[*i as usize],
        e => todo!("{:?}", e),
    }
}
