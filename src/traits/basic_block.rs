use id_arena::Id;
use rustc_hash::FxHashSet;
use std::fmt;

pub trait BasicBlock: Sized + fmt::Debug {
    fn preds(&self) -> &FxHashSet<Id<Self>>;
    fn succs(&self) -> &FxHashSet<Id<Self>>;
}

pub trait BasicBlockData<BB: BasicBlock> {
    fn get(&self, id: Id<BB>) -> &BB;
}

pub trait BasicBlockLayout<BB: BasicBlock> {
    fn order(&self) -> Box<dyn Iterator<Item = Id<BB>> + '_>;
}
