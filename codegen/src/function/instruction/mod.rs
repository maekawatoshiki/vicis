use crate::{
    function::{basic_block::BasicBlockId, slot::SlotId, Function},
    isa::TargetIsa,
    register::{Reg, VReg, VRegUsers},
};
use id_arena::Id;
use std::fmt::Debug;

pub type InstructionId<Data> = Id<Instruction<Data>>;

pub trait TargetInst: Clone + Debug {
    // TODO(FIXME): Too many methods?
    fn input_vregs_with_indexes(&self) -> Vec<(usize, VReg)>;
    fn input_vregs(&self) -> Vec<VReg>;
    fn output_vregs(&self) -> Vec<VReg>;
    fn all_vregs(&self) -> Vec<VReg>;
    fn input_regs(&self) -> Vec<Reg>;
    fn output_regs(&self) -> Vec<Reg>;
    fn all_regs(&self) -> Vec<Reg>;
    fn rewrite(&mut self, vreg: VReg, reg: Reg);
    fn replace_vreg(
        &mut self,
        self_id: InstructionId<Self>,
        users: &mut VRegUsers<Self>,
        from: VReg,
        to: VReg,
    );
    fn block_at(&self, i: usize) -> Option<BasicBlockId>;
    fn is_copy(&self) -> bool;
    fn is_call(&self) -> bool;
    fn is_phi(&self) -> bool;

    fn store_vreg_to_slot<T: TargetIsa>(
        f: &Function<T>,
        vreg: VReg,
        slot: SlotId,
        block: BasicBlockId,
    ) -> Instruction<Self>;
    fn load_from_slot<T: TargetIsa>(
        f: &Function<T>,
        vreg: VReg,
        slot: SlotId,
        block: BasicBlockId,
    ) -> Instruction<Self>;
}

#[derive(Debug, Clone)]
pub struct Instruction<Data: TargetInst> {
    pub id: Option<InstructionId<Data>>,
    pub data: Data,
    pub parent: BasicBlockId,
}

impl<Data: TargetInst> Instruction<Data> {
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

    pub fn store_vreg_to_slot<T: TargetIsa>(
        f: &Function<T>,
        vreg: VReg,
        slot: SlotId,
        block: BasicBlockId,
    ) -> Instruction<Data> {
        Data::store_vreg_to_slot(f, vreg, slot, block)
    }

    pub fn load_from_slot<T: TargetIsa>(
        f: &Function<T>,
        vreg: VReg,
        slot: SlotId,
        block: BasicBlockId,
    ) -> Instruction<Data> {
        Data::load_from_slot(f, vreg, slot, block)
    }
}
