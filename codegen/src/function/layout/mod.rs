use crate::function::{
    basic_block::BasicBlockId,
    instruction::{InstructionId, TargetInst},
};
use rustc_hash::FxHashMap;

pub struct Layout<Inst: TargetInst> {
    basic_blocks: FxHashMap<BasicBlockId, BasicBlockNode<Inst>>,
    instructions: FxHashMap<InstructionId<Inst>, InstructionNode<Inst>>,
    pub first_block: Option<BasicBlockId>,
    pub last_block: Option<BasicBlockId>,
}

pub struct BasicBlockNode<Inst: TargetInst> {
    _prev: Option<BasicBlockId>,
    next: Option<BasicBlockId>,
    first_inst: Option<InstructionId<Inst>>,
    last_inst: Option<InstructionId<Inst>>,
}

pub struct InstructionNode<Inst: TargetInst> {
    block: Option<BasicBlockId>,
    prev: Option<InstructionId<Inst>>,
    next: Option<InstructionId<Inst>>,
}

pub struct BasicBlockIter<'a, Inst: TargetInst> {
    layout: &'a Layout<Inst>,
    cur: Option<BasicBlockId>,
}

pub struct InstructionIter<'a, Inst: TargetInst> {
    layout: &'a Layout<Inst>,
    block: BasicBlockId,
    cur: Option<InstructionId<Inst>>,
}

impl<Inst: TargetInst> Default for Layout<Inst> {
    fn default() -> Self {
        Self {
            basic_blocks: FxHashMap::default(),
            instructions: FxHashMap::default(),
            first_block: None,
            last_block: None,
        }
    }
}

impl<Inst: TargetInst> Layout<Inst> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn block_iter(&self) -> BasicBlockIter<Inst> {
        BasicBlockIter {
            layout: self,
            cur: self.first_block,
        }
    }

    pub fn inst_iter(&self, block: BasicBlockId) -> InstructionIter<Inst> {
        InstructionIter {
            layout: self,
            block,
            cur: self.basic_blocks[&block].first_inst,
        }
    }

    pub fn next_block_of(&self, block: BasicBlockId) -> Option<BasicBlockId> {
        self.basic_blocks[&block].next
    }

    pub fn append_block(&mut self, block: BasicBlockId) {
        self.basic_blocks.entry(block).or_insert(BasicBlockNode {
            _prev: self.last_block,
            next: None,
            first_inst: None,
            last_inst: None,
        });

        if let Some(last_block) = self.last_block {
            self.basic_blocks.get_mut(&last_block).unwrap().next = Some(block);
            self.basic_blocks.get_mut(&block).unwrap()._prev = Some(last_block);
        }

        self.last_block = Some(block);

        if self.first_block.is_none() {
            self.first_block = Some(block)
        }
    }

    pub fn last_inst_of(&self, block: BasicBlockId) -> Option<InstructionId<Inst>> {
        self.basic_blocks[&block].last_inst
    }

    pub fn prev_inst_of(&self, inst: InstructionId<Inst>) -> Option<InstructionId<Inst>> {
        self.instructions[&inst].prev
    }

    pub fn next_inst_of(&self, inst: InstructionId<Inst>) -> Option<InstructionId<Inst>> {
        self.instructions[&inst].next
    }

    pub fn insert_inst_at_start(&mut self, inst: InstructionId<Inst>, block: BasicBlockId) {
        self.instructions
            .entry(inst)
            .or_insert(InstructionNode {
                prev: None,
                next: None,
                block: Some(block),
            })
            .block = Some(block);

        if let Some(first_inst) = self.basic_blocks[&block].first_inst {
            self.instructions.get_mut(&first_inst).unwrap().prev = Some(inst);
            self.instructions.get_mut(&inst).unwrap().next = Some(first_inst);
        }

        self.basic_blocks.get_mut(&block).unwrap().first_inst = Some(inst);

        if self.basic_blocks[&block].last_inst.is_none() {
            self.basic_blocks.get_mut(&block).unwrap().last_inst = Some(inst);
        }
    }

    pub fn insert_inst_before(
        &mut self,
        before: InstructionId<Inst>,
        inst: InstructionId<Inst>,
        block: BasicBlockId,
    ) {
        {
            let prev = self.instructions[&before].prev;
            self.instructions
                .entry(inst)
                .or_insert(InstructionNode {
                    prev,
                    next: Some(before),
                    block: Some(block),
                })
                .block = Some(block);
        }

        if let Some(prev) = self.instructions[&before].prev {
            self.instructions.get_mut(&prev).unwrap().next = Some(inst);
        }

        self.instructions.get_mut(&before).unwrap().prev = Some(inst);

        if self.basic_blocks[&block].first_inst == Some(before) {
            self.basic_blocks.get_mut(&block).unwrap().first_inst = Some(inst);
        }
    }

    pub fn insert_inst_after(
        &mut self,
        after: InstructionId<Inst>,
        inst: InstructionId<Inst>,
        block: BasicBlockId,
    ) {
        {
            let next = self.instructions[&after].next;
            self.instructions
                .entry(inst)
                .or_insert(InstructionNode {
                    prev: Some(after),
                    next,
                    block: Some(block),
                })
                .block = Some(block);
        }

        if let Some(next) = self.instructions[&after].next {
            self.instructions.get_mut(&next).unwrap().prev = Some(inst);
        }

        self.instructions.get_mut(&after).unwrap().next = Some(inst);

        // if self.basic_blocks[&block].first_inst == Some(before) {
        //     self.basic_blocks.get_mut(&block).unwrap().first_inst = Some(inst);
        // }
    }

    pub fn append_inst(&mut self, inst: InstructionId<Inst>, block: BasicBlockId) {
        self.instructions
            .entry(inst)
            .or_insert(InstructionNode {
                prev: self.basic_blocks[&block].last_inst,
                next: None,
                block: Some(block),
            })
            .block = Some(block);

        if let Some(last_inst) = self.basic_blocks[&block].last_inst {
            self.instructions.get_mut(&last_inst).unwrap().next = Some(inst);
            self.instructions.get_mut(&inst).unwrap().prev = Some(last_inst);
        }

        self.basic_blocks.get_mut(&block).unwrap().last_inst = Some(inst);

        if self.basic_blocks[&block].first_inst.is_none() {
            self.basic_blocks.get_mut(&block).unwrap().first_inst = Some(inst);
        }
    }

    pub fn remove_inst(&mut self, inst: InstructionId<Inst>) -> Option<()> {
        let block = self.instructions[&inst].block?;
        let prev;
        let next;
        {
            let inst_node = &mut self.instructions.get_mut(&inst)?;
            prev = inst_node.prev;
            next = inst_node.next;
            inst_node.block = None;
            inst_node.prev = None;
            inst_node.next = None;
        }
        match prev {
            Some(prev) => self.instructions.get_mut(&prev)?.next = next,
            None => self.basic_blocks.get_mut(&block)?.first_inst = next,
        }
        match next {
            Some(next) => self.instructions.get_mut(&next)?.prev = prev,
            None => self.basic_blocks.get_mut(&block)?.last_inst = prev,
        }
        Some(())
    }
}

impl<'a, Inst: TargetInst> Iterator for BasicBlockIter<'a, Inst> {
    type Item = BasicBlockId;

    fn next(&mut self) -> Option<Self::Item> {
        let cur = self.cur?;
        self.cur = self.layout.basic_blocks[&cur].next;
        Some(cur)
    }
}

impl<'a, Inst: TargetInst> Iterator for InstructionIter<'a, Inst> {
    type Item = InstructionId<Inst>;

    fn next(&mut self) -> Option<Self::Item> {
        let cur = self.cur?;
        if Some(cur) == self.layout.basic_blocks[&self.block].last_inst {
            self.cur = None;
        } else {
            self.cur = self.layout.instructions[&cur].next;
        }
        Some(cur)
    }
}
