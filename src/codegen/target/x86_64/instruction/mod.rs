use super::register::GR32;
use crate::ir::instruction::InstructionId;
use either::Either;

#[derive(Debug)]
pub enum InstructionData {
    MOVri32 {
        dst: Either<GR32, InstructionId>,
        src: i32,
    },
}
