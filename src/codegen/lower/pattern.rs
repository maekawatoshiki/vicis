use crate::codegen::{
    function::{
        basic_block::BasicBlockId as MachBasicBlockId,
        data::Data as MachData,
        instruction::Instruction as MachInstruction,
        slot::{SlotId, Slots},
    },
    register::{VReg, VRegs},
    target::Target,
};
use crate::ir::{
    function::{
        basic_block::BasicBlockId as IrBasicBlockId,
        instruction::{Instruction, InstructionId},
        Data as IrData,
    },
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
    pub block_map: &'a FxHashMap<IrBasicBlockId, MachBasicBlockId>,
    pub cur_block: IrBasicBlockId,
}
