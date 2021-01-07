use super::module::name::Name;
use id_arena::Id;
use rustc_hash::FxHashSet;

pub type BasicBlockId = Id<BasicBlock>;

pub struct BasicBlock {
    pub name: Option<Name>,
    pub preds: FxHashSet<BasicBlockId>,
    pub succs: FxHashSet<BasicBlockId>,
}

impl BasicBlock {
    pub fn new() -> Self {
        Self {
            name: None,
            preds: FxHashSet::default(),
            succs: FxHashSet::default(),
        }
    }

    pub fn preds(&self) -> &FxHashSet<BasicBlockId> {
        &self.preds
    }

    pub fn preds_mut(&mut self) -> &mut FxHashSet<BasicBlockId> {
        &mut self.preds
    }

    pub fn succs(&self) -> &FxHashSet<BasicBlockId> {
        &self.succs
    }

    pub fn succs_mut(&mut self) -> &mut FxHashSet<BasicBlockId> {
        &mut self.succs
    }
}
