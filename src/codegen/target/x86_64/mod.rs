pub mod asm;
pub mod calling_conv;
pub mod instruction;
pub mod lower;
pub mod pass;
pub mod register;

use super::Target;
use crate::codegen::{
    calling_conv::CallingConv,
    module::Module,
    pass::regalloc,
    register::{Reg, RegUnit},
    target::x86_64,
};

#[derive(Copy, Clone)]
pub struct X86_64<CC: CallingConv> {
    lower: x86_64::lower::Lower,
    calling_conv: CC,
}

impl<CC: CallingConv> X86_64<CC> {
    pub fn new(calling_conv: CC) -> Self {
        Self {
            lower: x86_64::lower::Lower::new(),
            calling_conv,
        }
    }
}

impl<CC: CallingConv> Target for X86_64<CC> {
    type InstData = instruction::InstructionData;
    type Lower = x86_64::lower::Lower;
    type CallingConv = CC;

    fn lower(&self) -> &Self::Lower {
        &self.lower
    }

    fn calling_conv(&self) -> &Self::CallingConv {
        &self.calling_conv
    }

    fn module_pass(&self) -> Vec<fn(&mut Module<Self>)> {
        vec![
            regalloc::run_on_module,
            pass::simple_reg_coalescing::run_on_module,
            pass::eliminate_slot::run_on_module,
            pass::pro_epi_inserter::run_on_module,
        ]
    }

    fn to_reg_unit(&self, r: Reg) -> RegUnit {
        register::to_reg_unit(r)
    }
}
