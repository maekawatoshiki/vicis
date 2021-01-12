pub mod x86_64;

use crate::codegen::{
    calling_conv,
    function::instruction::InstructionData,
    lower,
    module::Module,
    register::{Reg, RegUnit, RegisterClass},
};

pub trait Target: Copy {
    type InstData: ::std::fmt::Debug + InstructionData;
    type RegClass: RegisterClass;
    type Lower: lower::Lower<Self>;
    type CallingConv: calling_conv::CallingConv<Self::RegClass>;

    fn module_pass() -> Vec<fn(&mut Module<Self>)>;
    fn to_reg_unit(r: Reg) -> RegUnit;
}
