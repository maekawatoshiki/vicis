use crate::codegen::{
    function::slot::SlotId,
    instruction::InstructionData as ID,
    register::{Reg, VReg},
};
// use crate::ir::instruction::InstructionId;
use either::Either;

#[derive(Debug)]
pub enum InstructionData {
    PUSH64 {
        r: Either<Reg, VReg>,
    },
    POP64 {
        r: Either<Reg, VReg>,
    },
    ADDr64i32 {
        r: Either<Reg, VReg>,
        imm: i32,
    },
    SUBr64i32 {
        r: Either<Reg, VReg>,
        imm: i32,
    },
    MOVrr32 {
        dst: Either<Reg, VReg>,
        src: Either<Reg, VReg>,
    },
    MOVrr64 {
        dst: Either<Reg, VReg>,
        src: Either<Reg, VReg>,
    },
    MOVri32 {
        dst: Either<Reg, VReg>,
        src: i32,
    },
    MOVrm32 {
        dst: Either<Reg, VReg>,
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

impl MemoryOperand {
    pub fn vregs(&self) -> Vec<VReg> {
        match self {
            Self::Slot(_) => vec![],
            Self::ImmReg(_, _) => vec![],
        }
    }
}

impl ID for InstructionData {
    fn input_vregs(&self) -> Vec<VReg> {
        match self {
            Self::PUSH64 {
                r: Either::Right(r),
            } => vec![*r],
            Self::POP64 {
                r: Either::Right(r),
            } => vec![*r],
            Self::ADDr64i32 {
                r: Either::Right(r),
                ..
            } => vec![*r],
            Self::SUBr64i32 {
                r: Either::Right(r),
                ..
            } => vec![*r],
            Self::MOVrr32 {
                src: Either::Right(src),
                ..
            } => vec![*src],
            Self::MOVrr64 {
                src: Either::Right(src),
                ..
            } => vec![*src],
            Self::MOVrm32 { src, .. } => src.vregs(),
            Self::MOVmi32 { .. } => {
                vec![]
            }
            _ => vec![],
        }
    }

    fn output_vregs(&self) -> Vec<VReg> {
        match self {
            Self::ADDr64i32 {
                r: Either::Right(r),
                ..
            } => vec![*r],
            Self::SUBr64i32 {
                r: Either::Right(r),
                ..
            } => vec![*r],
            Self::MOVrr32 {
                dst: Either::Right(dst),
                ..
            } => vec![*dst],
            Self::MOVrr64 {
                dst: Either::Right(dst),
                ..
            } => vec![*dst],
            Self::MOVrm32 {
                dst: Either::Right(dst),
                ..
            } => vec![*dst],
            Self::MOVmi32 { .. } => {
                vec![]
            }
            _ => vec![],
        }
    }
}
