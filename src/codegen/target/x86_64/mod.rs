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
pub struct X86_64<CC: CallingConv<register::RegClass>> {
    lower: x86_64::lower::Lower,
    calling_conv: CC,
}

impl<CC: CallingConv<register::RegClass>> X86_64<CC> {
    pub fn new(calling_conv: CC) -> Self {
        Self {
            lower: x86_64::lower::Lower::new(),
            calling_conv,
        }
    }
}

impl<CC: CallingConv<register::RegClass>> Target for X86_64<CC> {
    type InstData = instruction::InstructionData;
    type Lower = x86_64::lower::Lower;
    type RegClass = register::RegClass;
    type CallingConv = CC;

    fn module_pass() -> Vec<fn(&mut Module<Self>)> {
        vec![
            regalloc::run_on_module,
            pass::phi_elimination::run_on_module, // TODO: should be target independent
            pass::simple_reg_coalescing::run_on_module,
            pass::eliminate_slot::run_on_module,
            pass::pro_epi_inserter::run_on_module,
        ]
    }

    fn to_reg_unit(r: Reg) -> RegUnit {
        register::to_reg_unit(r)
    }
}
