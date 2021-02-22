use crate::codegen::{
    function::{basic_block::BasicBlockId, slot::SlotId, Function},
    isa::TargetIsa,
    register::{Reg, VReg, VRegUsers},
};
use id_arena::Id;
use std::fmt;

pub type InstructionId<Data> = Id<Instruction<Data>>;

pub trait InstructionData: Clone + fmt::Debug {
    fn input_vregs(&self) -> Vec<VReg>;
    fn output_vregs(&self) -> Vec<VReg>;
    fn input_regs(&self) -> Vec<Reg>;
    fn output_regs(&self) -> Vec<Reg>;
    fn rewrite(&mut self, vreg: VReg, reg: Reg);
    fn replace_vreg(
        &mut self,
        self_id: InstructionId<Self>,
        users: &mut VRegUsers<Self>,
        from: VReg,
        to: VReg,
    );
    fn is_copy(&self) -> bool;
}

pub trait InstructionInfo {
    type Data: InstructionData;
    fn store_vreg_to_slot<T: TargetIsa>(
        f: &Function<T>,
        vreg: VReg,
        slot: SlotId,
        block: BasicBlockId,
    ) -> Instruction<Self::Data>;
    fn load_from_slot<T: TargetIsa>(
        f: &Function<T>,
        vreg: VReg,
        slot: SlotId,
        block: BasicBlockId,
    ) -> Instruction<Self::Data>;
}

#[derive(Debug, Clone)]
pub struct Instruction<Data: InstructionData> {
    pub id: Option<InstructionId<Data>>,
    pub data: Data,
    pub parent: BasicBlockId,
}

impl<Data: InstructionData> Instruction<Data> {
    pub fn new(data: Data, parent: BasicBlockId) -> Self {
        Self {
            id: None,
            data,
            parent,
        }
    }

    pub fn replace_vreg(&mut self, users: &mut VRegUsers<Data>, from: VReg, to: VReg) {
        if let Some(id) = self.id {
            self.data.replace_vreg(id, users, from, to)
        }
    }
}
