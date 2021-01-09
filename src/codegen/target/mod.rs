pub mod x86_64;

use crate::codegen::{
    calling_conv,
    function::instruction::InstructionData,
    lower,
    module::Module,
    register::{Reg, RegUnit},
};

pub trait Target: Copy {
    type InstData: ::std::fmt::Debug + InstructionData;
    type Lower: lower::pattern::Lower<Self>;
    type CallingConv: calling_conv::CallingConv;

    fn lower(&self) -> &Self::Lower;
    fn calling_conv(&self) -> &Self::CallingConv;
    fn module_pass(&self) -> Vec<fn(&mut Module<Self>)>;
    fn to_reg_unit(&self, r: Reg) -> RegUnit;
}
