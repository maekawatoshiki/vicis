pub mod asm;
pub mod instruction;
pub mod lower;
pub mod pass;
pub mod register;

use super::TargetIsa;
use crate::codegen::{call_conv::CallConvKind, isa::x86_64, module::Module, pass::regalloc};
use anyhow::Result;
use vicis_core::ir::types::{self, ArrayType, CompoundType, Type, Types};

#[derive(Copy, Clone)]
pub struct X86_64;

impl TargetIsa for X86_64 {
    type InstInfo = instruction::InstructionInfo;
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

    fn type_size(types: &Types, ty: Type) -> u32 {
        match types.get(ty) {
            Some(ty) => match &*ty {
                CompoundType::Pointer(_) => 8,
                CompoundType::Array(ArrayType {
                    inner,
                    num_elements,
                }) => Self::type_size(types, *inner) * num_elements,
                CompoundType::Function(_) => 0,
                CompoundType::Struct(_) => todo!(),
                CompoundType::Metadata => todo!(),
                CompoundType::Alias(_) => todo!(),
            },
            None => match ty {
                types::VOID => 0,
                types::I1 => 1,
                types::I8 => 1,
                types::I16 => 2,
                types::I32 => 4,
                types::I64 => 8,
                _ => unreachable!(),
            },
        }
    }
}
