use id_arena::Id;

pub type BasicBlockId = Id<BasicBlock>;

pub struct BasicBlock {}

impl BasicBlock {
    pub fn new() -> Self {
        Self {}
    }
}
