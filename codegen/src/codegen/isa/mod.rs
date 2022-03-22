pub mod x86_64;

use crate::codegen::{
    call_conv::CallConvKind,
    function::instruction::TargetInst,
    lower,
    module::Module,
    register::{RegisterClass, RegisterInfo},
};
use anyhow::Result;
use vicis_core::ir::{
    module::data_layout::DataLayout,
    types::{Type, Types},
};

pub trait TargetIsa: Clone {
    type Inst: TargetInst;
    type RegClass: RegisterClass;
    type RegInfo: RegisterInfo;
    type Lower: lower::Lower<Self>;

    fn module_passes() -> Vec<for<'a, 'b> fn(&'b mut Module<'a, Self>) -> Result<()>>; // TODO: Implement a pass manager for machine modules
    fn default_call_conv() -> CallConvKind;
    fn type_size(types: &Types, ty: Type) -> u32; // TODO: FIXME: DataLayout can replace this.
    fn data_layout(&self) -> &DataLayout;
}
