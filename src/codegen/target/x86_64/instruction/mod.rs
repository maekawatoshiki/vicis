use super::register::{GR32, GR64};
use crate::codegen::{
    function::slot::SlotId,
    register::{Reg, VReg},
};
// use crate::ir::instruction::InstructionId;
use either::Either;

#[derive(Debug)]
pub enum InstructionData {
    PUSH64 {
        r: Either<GR64, VReg>,
    },
    POP64 {
        r: Either<GR64, VReg>,
    },
    ADDr64i32 {
        r: Either<GR64, VReg>,
        imm: i32,
    },
    SUBr64i32 {
        r: Either<GR64, VReg>,
        imm: i32,
    },
    MOVrr32 {
        dst: Either<GR32, VReg>,
        src: Either<GR32, VReg>,
    },
    MOVri32 {
        dst: Either<GR32, VReg>,
        src: i32,
    },
    MOVrm32 {
        dst: Either<GR32, VReg>,
        src: MemoryOperand,
    },
    MOVmi32 {
        dst: MemoryOperand,
        src: i32,
    },
    RET,
}

#[derive(Debug)]
pub enum MemoryOperand {
    Slot(SlotId),
    ImmReg(i32, Reg),
}

impl InstructionData {
    pub fn mem_ops(&self) -> &[MemoryOperand] {
        match self {
            Self::MOVmi32 { dst, .. } => ::std::slice::from_ref(dst),
            Self::MOVrm32 { src, .. } => ::std::slice::from_ref(src),
            _ => &mut [],
        }
    }

    pub fn mem_ops_mut(&mut self) -> &mut [MemoryOperand] {
        match self {
            Self::MOVmi32 { dst, .. } => ::std::slice::from_mut(dst),
            Self::MOVrm32 { src, .. } => ::std::slice::from_mut(src),
            _ => &mut [],
        }
    }
}
