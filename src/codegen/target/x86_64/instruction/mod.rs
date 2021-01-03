use super::register::{GR32, GR64};
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
    RET,
}
