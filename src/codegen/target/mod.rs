pub mod x86_64;

use crate::codegen::{
    instruction::InstructionData,
    lower,
    module::Module,
    register::{Reg, RegUnit},
};

pub trait Target: Copy {
    type InstData: ::std::fmt::Debug + InstructionData;
    type Lower: lower::pattern::Lower<Self>;

    fn lower(&self) -> &Self::Lower;
    fn module_pass(&self) -> Vec<fn(&mut Module<Self>)>;
    fn to_reg_unit(&self, r: Reg) -> RegUnit;
}
