pub mod x86_64;

use crate::codegen::{inst_selection, instruction::Instruction as MachInstruction};
use crate::ir::{function::Data, instruction::Instruction};

pub trait Target {
    type InstData: ::std::fmt::Debug;

    fn select_patterns() -> Vec<
        fn(inst_selection::Context<Self::InstData>) -> Option<Vec<MachInstruction<Self::InstData>>>,
    >;
}
