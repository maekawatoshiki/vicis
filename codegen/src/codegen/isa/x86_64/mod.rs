pub mod asm;
pub mod instruction;
pub mod lower;
pub mod pass;
pub mod register;

use super::TargetIsa;
use crate::codegen::{call_conv::CallConvKind, isa::x86_64, module::Module, pass::regalloc};
use anyhow::Result;
use vicis_core::ir::module::data_layout::DataLayout;

#[derive(Clone)]
pub struct X86_64 {
    data_layout: DataLayout,
}

impl Default for X86_64 {
    fn default() -> Self {
        Self {
            data_layout: DataLayout("".to_string()),
        }
    }
}

impl TargetIsa for X86_64 {
    type Inst = instruction::InstructionInfo;
    type Lower = x86_64::lower::Lower;
    type RegClass = register::RegClass;
    type RegInfo = register::RegInfo;

    fn module_passes() -> Vec<for<'a, 'b> fn(&'b mut Module<'a, Self>) -> Result<()>> {
        vec![
            regalloc::run_on_module,
            pass::phi_elimination::run_on_module, // TODO: should be target independent
            pass::simple_reg_coalescing::run_on_module,
            pass::eliminate_slot::run_on_module,
            pass::pro_epi_inserter::run_on_module,
        ]
    }

    fn default_call_conv() -> CallConvKind {
        CallConvKind::SystemV
    }

    fn data_layout(&self) -> &DataLayout {
        &self.data_layout
    }
}
