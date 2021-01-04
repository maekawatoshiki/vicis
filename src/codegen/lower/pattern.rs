use crate::codegen::{
    function::data::Data as MachData, instruction::Instruction as MachInstruction,
};
use crate::ir::{function::Data as IrData, instruction::Instruction};

pub trait Lower<InstData> {
    fn lower(
        &self,
        ctx: &mut LoweringContext<InstData>,
        inst: &Instruction,
    ) -> Vec<MachInstruction<InstData>>;
}

pub struct LoweringContext<'a, InstData> {
    pub ir_data: &'a IrData,
    pub mach_data: &'a mut MachData<InstData>,
}
