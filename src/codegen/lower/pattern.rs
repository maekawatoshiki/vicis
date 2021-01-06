use crate::codegen::{
    function::{
        data::Data as MachData,
        slot::{SlotId, Slots},
    },
    instruction::Instruction as MachInstruction,
    register::{VReg, VRegs},
    target::Target,
};
use crate::ir::{
    function::Data as IrData,
    instruction::{Instruction, InstructionId},
    types::Types,
};
use rustc_hash::FxHashMap;

pub trait Lower<T: Target> {
    fn lower(&self, ctx: &mut LoweringContext<T>, inst: &Instruction);
}

pub struct LoweringContext<'a, T: Target> {
    pub ir_data: &'a IrData,
    pub mach_data: &'a mut MachData<T::InstData>,
    pub slots: &'a mut Slots<T>,
    pub inst_id_to_slot_id: &'a mut FxHashMap<InstructionId, SlotId>,
    pub inst_seq: &'a mut Vec<MachInstruction<T::InstData>>,
    pub types: &'a Types,
    pub vregs: &'a mut VRegs,
    pub inst_id_to_vreg: &'a mut FxHashMap<InstructionId, VReg>,
}
