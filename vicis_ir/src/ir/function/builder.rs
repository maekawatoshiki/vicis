use super::{basic_block::BasicBlockId, Function};

pub struct Builder<'a> {
    func: &'a mut Function,
    cur_block: Option<BasicBlockId>,
}

impl<'a> Builder<'a> {
    pub fn new(func: &'a mut Function) -> Self {
        Self {
            func,
            cur_block: None,
        }
    }

    pub fn create_block(&mut self) -> BasicBlockId {
        self.func.data.create_block()
    }

    pub fn append_block(&mut self, block: BasicBlockId) {
        self.func.layout.append_block(block);
        self.cur_block = Some(block);
    }
}
