use super::register::{GR32, GR64};
use crate::codegen::{function::slot::SlotId, register::Reg};
use crate::ir::instruction::InstructionId;
use either::Either;

#[derive(Debug)]
pub enum InstructionData {
    PUSH64 {
        r: Either<GR64, InstructionId>,
    },
    POP64 {
        r: Either<GR64, InstructionId>,
    },
    ADDr64i32 {
        r: Either<GR64, InstructionId>,
        imm: i32,
    },
    SUBr64i32 {
        r: Either<GR64, InstructionId>,
        imm: i32,
    },
    MOVri32 {
        dst: Either<GR32, InstructionId>,
        src: i32,
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
            _ => &mut [],
        }
    }

    pub fn mem_ops_mut(&mut self) -> &mut [MemoryOperand] {
        match self {
            Self::MOVmi32 { dst, .. } => ::std::slice::from_mut(dst),
            _ => &mut [],
        }
    }
}
