pub mod asm;
pub mod instruction;
pub mod lower;
pub mod pass;
pub mod register;

use super::Target;
use crate::codegen::{
    call_conv::CallConvKind,
    module::Module,
    pass::regalloc,
    register::{Reg, RegUnit},
    target::x86_64,
};

#[derive(Copy, Clone)]
pub struct X86_64;

impl Target for X86_64 {
    type InstData = instruction::InstructionData;
    type Lower = x86_64::lower::Lower;
    type RegClass = register::RegClass;

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

    fn default_call_conv() -> CallConvKind {
        CallConvKind::SystemV
    }
}
