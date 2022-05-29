use crate::{
    function::{
        basic_block::{BasicBlock, BasicBlockId},
        instruction::{Instruction, InstructionId, TargetInst},
    },
    register::{RegUnit, VRegUsers, VRegs},
};
use id_arena::Arena;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

pub struct Data<Inst: TargetInst> {
    // pub values: Arena<Value>,
    pub instructions: Arena<Instruction<Inst>>,
    pub basic_blocks: Arena<BasicBlock>,
    pub vregs: VRegs,
    pub vreg_users: VRegUsers<Inst>,
    pub used_csr: HashSet<RegUnit>,
}

impl<Inst: TargetInst> Default for Data<Inst> {
    fn default() -> Self {
        Self {
            // values: Arena::new(),
            instructions: Arena::new(),
            basic_blocks: Arena::new(),
            vregs: VRegs::new(),
            vreg_users: VRegUsers::new(),
            used_csr: HashSet::default(),
        }
    }
}

impl<Inst: TargetInst> Data<Inst> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_block(&mut self) -> BasicBlockId {
        self.basic_blocks.alloc(BasicBlock::new())
    }

    pub fn create_inst(&mut self, mut inst: Instruction<Inst>) -> InstructionId<Inst> {
        // TODO: FIXME: Refine code
        struct ReadWrite(bool, bool);
        let mut m = HashMap::default();
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

    pub fn inst_ref(&self, id: InstructionId<Inst>) -> &Instruction<Inst> {
        &self.instructions[id]
    }

    pub fn inst_ref_mut(&mut self, id: InstructionId<Inst>) -> &mut Instruction<Inst> {
        &mut self.instructions[id]
    }
}
