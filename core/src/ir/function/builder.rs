use crate::ir::value::{Value, ValueId};

use super::{basic_block::BasicBlockId, instruction::builder::Builder as InstBuilder, Function};
use rustc_hash::FxHashSet;

pub struct Builder<'a> {
    ctx: Context,
    pub(super) func: &'a mut Function,
    pub(super) cur_block: Option<BasicBlockId>,
}

#[derive(Default)]
struct Context {
    is_inserted: FxHashSet<BasicBlockId>,
}

impl<'a> Builder<'a> {
    pub fn new(func: &'a mut Function) -> Self {
        Self {
            ctx: Context::default(),
            func,
            cur_block: None,
        }
    }

    pub fn create_block(&mut self) -> BasicBlockId {
        self.func.data.create_block()
    }

    pub fn switch_to_block(&mut self, block: BasicBlockId) {
        self.cur_block = Some(block);
        self.ensure_inserted_block(block);
    }

    pub fn ensure_inserted_block(&mut self, block: BasicBlockId) {
        if self.ctx.is_inserted(block) {
            return;
        }
        self.func.layout.append_block(block);
        self.ctx.set_as_inserted(block);
    }

    pub fn inst<'short>(&'short mut self) -> InstBuilder<'a, 'short> {
        InstBuilder::new(self)
    }

    pub fn value<T: Into<Value>>(&mut self, val: T) -> ValueId {
        self.func.data.create_value(val.into())
    }
}

impl Context {
    fn set_as_inserted(&mut self, block: BasicBlockId) {
        self.is_inserted.insert(block);
    }

    fn is_inserted(&self, block: BasicBlockId) -> bool {
        self.is_inserted.contains(&block)
    }
}
