use super::register::{GR32, GR64};
use crate::codegen::function::slot::SlotId;
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

type Reg = (); // TODO

#[derive(Debug)]
pub enum MemoryOperand {
    Slot(SlotId),
    ImmReg(i32, Reg),
    // BaseImm(X, i32),
}
