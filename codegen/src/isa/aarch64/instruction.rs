use crate::{
    function::{
        basic_block::BasicBlockId,
        instruction::{Instruction, InstructionId, TargetInst},
        slot::SlotId,
        Function,
    },
    isa::TargetIsa,
    register::{Reg, VReg, VRegUsers},
};

#[derive(Clone, Debug)]
pub struct InstructionData {
    // pub opcode: Opcode,
// pub operands: Vec<Operand>,
}

impl TargetInst for InstructionData {
    fn input_vregs_with_indexes(&self) -> Vec<(usize, VReg)> {
        todo!()
    }
    fn input_vregs(&self) -> Vec<VReg> {
        todo!()
    }
    fn output_vregs(&self) -> Vec<VReg> {
        todo!()
    }
    fn all_vregs(&self) -> Vec<VReg> {
        todo!()
    }
    fn input_regs(&self) -> Vec<Reg> {
        todo!()
    }
    fn output_regs(&self) -> Vec<Reg> {
        todo!()
    }
    fn all_regs(&self) -> Vec<Reg> {
        todo!()
    }
    fn rewrite(&mut self, _vreg: VReg, _reg: Reg) {
        todo!()
    }
    fn replace_vreg(
        &mut self,
        _self_id: InstructionId<Self>,
        _users: &mut VRegUsers<Self>,
        _from: VReg,
        _to: VReg,
    ) {
        todo!()
    }
    fn block_at(&self, _i: usize) -> Option<BasicBlockId> {
        todo!()
    }
    fn is_copy(&self) -> bool {
        todo!()
    }
    fn is_call(&self) -> bool {
        todo!()
    }
    fn is_phi(&self) -> bool {
        todo!()
    }

    fn store_vreg_to_slot<T: TargetIsa>(
        _f: &Function<T>,
        _vreg: VReg,
        _slot: SlotId,
        _block: BasicBlockId,
    ) -> Instruction<Self> {
        todo!()
    }
    fn load_from_slot<T: TargetIsa>(
        _f: &Function<T>,
        _vreg: VReg,
        _slot: SlotId,
        _block: BasicBlockId,
    ) -> Instruction<Self> {
        todo!()
    }
}
