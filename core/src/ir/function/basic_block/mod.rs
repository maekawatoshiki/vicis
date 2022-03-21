use crate::{ir::module::name::Name, traits::basic_block::BasicBlock as BB};
use id_arena::Id;
use rustc_hash::FxHashSet;

pub type BasicBlockId = Id<BasicBlock>;

#[derive(Debug, Default)]
pub struct BasicBlock {
    pub name: Option<Name>,
    pub preds: FxHashSet<BasicBlockId>,
    pub succs: FxHashSet<BasicBlockId>,
}

impl BasicBlock {
    pub fn new() -> Self {
        Self::default()
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

impl BB for BasicBlock {
    fn preds(&self) -> &FxHashSet<Id<Self>> {
        &self.preds
    }

    fn succs(&self) -> &FxHashSet<Id<Self>> {
        &self.succs
    }
}
