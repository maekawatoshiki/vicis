pub mod asm;
pub mod instruction;
pub mod lower;
// pub mod pass;
pub mod register;

use super::TargetIsa;
use crate::{call_conv::CallConvKind, module::Module};
use anyhow::Result;
use vicis_core::ir::module::data_layout::DataLayout;

#[derive(Clone)]
pub struct Aarch64 {
    data_layout: DataLayout,
}

impl Default for Aarch64 {
    fn default() -> Self {
        Self {
            data_layout: DataLayout("".to_string()),
        }
    }
}

impl TargetIsa for Aarch64 {
    type Inst = instruction::InstructionData;
    type Lower = lower::Lower;
    type RegClass = register::RegClass;
    type RegInfo = register::RegInfo;

    fn module_passes() -> Vec<for<'a, 'b> fn(&'b mut Module<'a, Self>) -> Result<()>> {
        todo!()
        // vec![
        //     regalloc::run_on_module,
        //     // pass::phi_elimination::run_on_module, // TODO: should be target independent
        //     // pass::simple_reg_coalescing::run_on_module,
        //     // pass::eliminate_slot::run_on_module,
        //     // pass::pro_epi_inserter::run_on_module,
        // ]
    }

    fn default_call_conv() -> CallConvKind {
        CallConvKind::AAPCS64
    }

    fn data_layout(&self) -> &DataLayout {
        &self.data_layout
    }
}
