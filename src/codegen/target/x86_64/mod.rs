pub mod instruction;

use super::Target;

pub struct X86_64 {}

impl Target for X86_64 {
    type InstData = instruction::InstructionData;

    fn select_patterns() -> Vec<()> {
        vec![]
    }
}
