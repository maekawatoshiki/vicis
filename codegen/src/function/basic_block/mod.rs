use id_arena::Id;
use rustc_hash::FxHashSet;

pub type BasicBlockId = Id<BasicBlock>;

#[derive(Default)]
pub struct BasicBlock {
    pub preds: FxHashSet<BasicBlockId>,
    pub succs: FxHashSet<BasicBlockId>,
}

impl BasicBlock {
    pub fn new() -> Self {
        Self::default()
    }
}
