pub mod asm;
pub mod instruction;
pub mod lower;
// pub mod pass;
pub mod register;

use super::TargetIsa;
use crate::{
    codegen::{call_conv::CallConvKind, module::Module},
    ir::types::{ArrayType, Type, TypeId, Types},
};
use anyhow::Result;

#[derive(Copy, Clone)]
pub struct MIPS32;

impl TargetIsa for MIPS32 {
    type InstInfo = instruction::InstructionInfo;
    type Lower = lower::Lower;
    type RegClass = register::RegClass;
    type RegInfo = register::RegInfo;

    fn module_pass_list() -> Vec<fn(&mut Module<Self>) -> Result<()>> {
        vec![
            // regalloc::run_on_module,
            // pass::phi_elimination::run_on_module, // TODO: should be target independent
            // pass::simple_reg_coalescing::run_on_module,
            // pass::eliminate_slot::run_on_module,
            // pass::pro_epi_inserter::run_on_module,
        ]
    }

    fn default_call_conv() -> CallConvKind {
        CallConvKind::MIPS
    }

    fn type_size(types: &Types, ty: TypeId) -> u32 {
        match &*types.get(ty) {
            Type::Void => 0,
            Type::Int(n) => *n / 8,
            Type::Pointer(_) => 4,
            Type::Array(ArrayType {
                inner,
                num_elements,
            }) => Self::type_size(types, *inner) * num_elements,
            Type::Function(_) => 0,
            Type::Struct(_) => todo!(),
        }
    }
}
