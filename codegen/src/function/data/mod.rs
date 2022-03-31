use crate::{
    function::{
        basic_block::{BasicBlock, BasicBlockId},
        instruction::{Instruction, InstructionData, InstructionId},
    },
    register::{VRegUsers, VRegs},
};
use id_arena::Arena;
use rustc_hash::FxHashMap;

pub struct Data<InstData: InstructionData> {
    // pub values: Arena<Value>,
    pub instructions: Arena<Instruction<InstData>>,
    pub basic_blocks: Arena<BasicBlock>,
    pub vregs: VRegs,
    pub vreg_users: VRegUsers<InstData>,
}

impl<InstData: InstructionData> Default for Data<InstData> {
    fn default() -> Self {
        Self {
            // values: Arena::new(),
            instructions: Arena::new(),
            basic_blocks: Arena::new(),
            vregs: VRegs::new(),
            vreg_users: VRegUsers::new(),
        }
    }
}

impl<InstData: InstructionData> Data<InstData> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_block(&mut self) -> BasicBlockId {
        self.basic_blocks.alloc(BasicBlock::new())
    }

    pub fn create_inst(&mut self, mut inst: Instruction<InstData>) -> InstructionId<InstData> {
        // TODO: FIXME: Refine code
        struct ReadWrite(bool, bool);
        let mut m = FxHashMap::default();
        for v in inst.data.input_vregs() {
            m.entry(v).or_insert(ReadWrite(false, false)).0 = true;
        }
        for v in inst.data.output_vregs() {
            m.entry(v).or_insert(ReadWrite(false, false)).1 = true;
        }
        let id = self.instructions.alloc_with_id(|id| {
            inst.id = Some(id);
            inst
        });
        for (v, ReadWrite(read, write)) in m {
            self.vreg_users.add_use(v, id, read, write)
        }
        id
    }

    pub fn block_ref(&self, id: BasicBlockId) -> &BasicBlock {
        &self.basic_blocks[id]
    }

    // TODO: Is this the right way?
    pub fn block_ref_mut(&mut self, id: BasicBlockId) -> &mut BasicBlock {
        &mut self.basic_blocks[id]
    }

    pub fn inst_ref(&self, id: InstructionId<InstData>) -> &Instruction<InstData> {
        &self.instructions[id]
    }

    pub fn inst_ref_mut(&mut self, id: InstructionId<InstData>) -> &mut Instruction<InstData> {
        &mut self.instructions[id]
    }
}
