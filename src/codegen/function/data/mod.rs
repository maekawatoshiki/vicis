use crate::codegen::{
    basic_block::{BasicBlock, BasicBlockId},
    instruction::{Instruction, InstructionId},
};
use id_arena::Arena;

pub struct Data<InstData> {
    // pub values: Arena<Value>,
    pub instructions: Arena<Instruction<InstData>>,
    pub basic_blocks: Arena<BasicBlock>,
}

impl<InstData> Data<InstData> {
    pub fn new() -> Self {
        Self {
            // values: Arena::new(),
            instructions: Arena::new(),
            basic_blocks: Arena::new(),
        }
    }

    pub fn create_block(&mut self) -> BasicBlockId {
        self.basic_blocks.alloc(BasicBlock::new())
    }

    pub fn create_inst(&mut self, mut inst: Instruction<InstData>) -> InstructionId<InstData> {
        let id = self.instructions.alloc_with_id(|id| {
            inst.id = Some(id);
            inst
        });
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
