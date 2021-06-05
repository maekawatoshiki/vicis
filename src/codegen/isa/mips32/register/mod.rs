use crate::{
    codegen::{
        call_conv::CallConvKind,
        register::{Reg, RegUnit, RegisterClass, RegisterInfo},
    },
    ir::types::{Type, TypeId, Types},
};
use std::fmt;

pub struct RegInfo;

pub enum GR {
    ZERO,
    AT,
    V0,
    V1,
    A0,
    A1,
    A2,
    A3,
    T0,
    T1,
    T2,
    T3,
    T4,
    T5,
    T6,
    T7,
    T8,
    T9,
    S0,
    S1,
    S2,
    S3,
    S4,
    S5,
    S6,
    S7,
    K0,
    K1,
    GP,
    SP,
    FP,
    RA,
}

pub enum RegClass {
    GR,
}

impl From<GR> for Reg {
    fn from(r: GR) -> Self {
        Reg(RegClass::GR as u16, r as u16)
    }
}

impl From<GR> for RegUnit {
    fn from(r: GR) -> Self {
        RegUnit(RegClass::GR as u16, r as u16)
    }
}

const ARG_REGS: [RegUnit; 4] = [
    RegUnit(RegClass::GR as u16, GR::A0 as u16),
    RegUnit(RegClass::GR as u16, GR::A1 as u16),
    RegUnit(RegClass::GR as u16, GR::A2 as u16),
    RegUnit(RegClass::GR as u16, GR::A3 as u16),
];

impl RegisterInfo for RegInfo {
    fn arg_reg_list(cc: &CallConvKind) -> &'static [RegUnit] {
        match cc {
            CallConvKind::MIPS => &ARG_REGS,
            CallConvKind::SystemV => panic!(),
        }
    }

    fn to_reg_unit(r: Reg) -> RegUnit {
        match r {
            Reg(/*GR*/ 0, x) => RegUnit(RegClass::GR as u16, x),
            _ => panic!(),
        }
    }
}

impl RegisterClass for RegClass {
    fn for_type(types: &Types, id: TypeId) -> Self {
        match &*types.get(id) {
            Type::Int(32) => Self::GR,
            Type::Int(64) => Self::GR,
            Type::Pointer(_) => Self::GR,
            _ => todo!(),
        }
    }

    fn gpr_list(&self) -> Vec<Reg> {
        match self {
            // TODO: Add more general-purpose registers
            RegClass::GR => vec![
                GR::T0,
                GR::T1,
                GR::T2,
                GR::T3,
                GR::T4,
                GR::T5,
                GR::T6,
                GR::T7,
                GR::T8,
                GR::T9,
            ]
            .into_iter()
            .map(|r| r.into())
            .collect(),
        }
    }

    fn apply_for(&self, ru: RegUnit) -> Reg {
        match self {
            Self::GR => Reg(RegClass::GR as u16, ru.1),
        }
    }
}

pub fn to_reg_unit(r: Reg) -> RegUnit {
    match r {
        Reg(/*GR*/ 0, x) => RegUnit(0, x),
        _ => panic!(),
    }
}

impl fmt::Debug for GR {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::ZERO => "zero",
                Self::AT => "at",
                Self::V0 => "v0",
                Self::V1 => "v1",
                Self::A0 => "a0",
                Self::A1 => "a1",
                Self::A2 => "a2",
                Self::A3 => "a3",
                Self::T0 => "t0",
                Self::T1 => "t1",
                Self::T2 => "t2",
                Self::T3 => "t3",
                Self::T4 => "t4",
                Self::T5 => "t5",
                Self::T6 => "t6",
                Self::T7 => "t7",
                Self::T8 => "t8",
                Self::T9 => "t9",
                Self::S0 => "s0",
                Self::S1 => "s1",
                Self::S2 => "s2",
                Self::S3 => "s3",
                Self::S4 => "s4",
                Self::S5 => "s5",
                Self::S6 => "s6",
                Self::S7 => "s7",
                Self::K0 => "k0",
                Self::K1 => "k1",
                Self::GP => "gp",
                Self::SP => "sp",
                Self::FP => "fp",
                Self::RA => "ra",
            }
        )
    }
}

impl fmt::Display for GR {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn reg_to_str(r: &Reg) -> &'static str {
    let gr = [
        "zero", "at", "v0", "v1", "a0", "a1", "a2", "a3", "t0", "t1", "t2", "t3", "t4", "t5", "t6",
        "t7", "t8", "t9", "s0", "s1", "s2", "s3", "s4", "s5", "s6", "s7", "k0", "k1", "gp", "sp",
        "fp", "ra",
    ];
    match r {
        Reg(0, i) => gr[*i as usize],
        e => todo!("{:?}", e),
    }
}
