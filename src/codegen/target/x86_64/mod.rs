pub mod asm;
pub mod inst_selection;
pub mod instruction;
pub mod pass;
pub mod register;

use super::Target;
use crate::codegen::{
    inst_selection::Context, instruction::Instruction as MachInstruction,
    target::x86_64::instruction::InstructionData,
};
use crate::ir::{function::Data, instruction::Instruction};

pub struct X86_64;

impl Target for X86_64 {
    type InstData = instruction::InstructionData;

    fn select_patterns(
    ) -> Vec<fn(Context<Self::InstData>) -> Option<Vec<MachInstruction<InstructionData>>>> {
        vec![inst_selection::ret]
    }
}
