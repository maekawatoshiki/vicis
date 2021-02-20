pub mod x86_64;

use crate::{
    codegen::{
        call_conv::CallConvKind,
        function::instruction::InstructionData,
        lower,
        module::Module,
        register::{RegisterClass, RegisterInfo},
    },
    ir::types::{TypeId, Types},
};
use anyhow::Result;

pub trait TargetIsa: Copy {
    type InstData: InstructionData;
    type RegClass: RegisterClass;
    type RegInfo: RegisterInfo;
    type Lower: lower::Lower<Self>;

    fn module_pass_list() -> Vec<fn(&mut Module<Self>) -> Result<()>>;
    fn default_call_conv() -> CallConvKind;
    fn type_size(types: &Types, ty: TypeId) -> u32;
}
