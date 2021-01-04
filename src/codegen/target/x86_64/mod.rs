pub mod asm;
pub mod instruction;
pub mod lower;
pub mod pass;
pub mod register;

use super::Target;
use crate::codegen::target::x86_64;

#[derive(Copy, Clone)]
pub struct X86_64 {
    lower: x86_64::lower::Lower,
}

impl X86_64 {
    pub fn new() -> Self {
        Self {
            lower: x86_64::lower::Lower::new(),
        }
    }
}

impl Target for X86_64 {
    type InstData = instruction::InstructionData;
    type Lower = x86_64::lower::Lower;

    fn lower(&self) -> &Self::Lower {
        &self.lower
    }
}
