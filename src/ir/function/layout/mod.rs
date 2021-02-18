use crate::ir::function::{basic_block::BasicBlockId, instruction::InstructionId};
use rustc_hash::FxHashMap;

pub struct Layout {
    basic_blocks: FxHashMap<BasicBlockId, BasicBlockNode>,
    instructions: FxHashMap<InstructionId, InstructionNode>,
    pub first_block: Option<BasicBlockId>,
    pub last_block: Option<BasicBlockId>,
}

#[derive(Debug)]
pub struct BasicBlockNode {
    prev: Option<BasicBlockId>,
    next: Option<BasicBlockId>,
    first_inst: Option<InstructionId>,
    last_inst: Option<InstructionId>,
}

#[derive(Debug)]
pub struct InstructionNode {
    block: Option<BasicBlockId>,
    prev: Option<InstructionId>,
    next: Option<InstructionId>,
}

pub struct BasicBlockIter<'a> {
    layout: &'a Layout,
    cur: Option<BasicBlockId>,
}

pub struct InstructionIter<'a> {
    layout: &'a Layout,
    block: BasicBlockId,
    head: Option<InstructionId>,
    tail: Option<InstructionId>,
}

impl Layout {
    pub fn new() -> Self {
        Self {
            basic_blocks: FxHashMap::default(),
            instructions: FxHashMap::default(),
            first_block: None,
            last_block: None,
        }
    }

    pub fn block_node(&self, id: BasicBlockId) -> &BasicBlockNode {
        &self.basic_blocks[&id]
    }

    pub fn block_iter<'a>(&'a self) -> BasicBlockIter<'a> {
        BasicBlockIter {
            layout: self,
            cur: self.first_block,
        }
    }

    pub fn inst_iter<'a>(&'a self, block: BasicBlockId) -> InstructionIter<'a> {
        InstructionIter {
            layout: self,
            block,
            head: self.basic_blocks[&block].first_inst,
            tail: self.basic_blocks[&block].last_inst,
        }
    }

    pub fn next_block_of(&self, block: BasicBlockId) -> Option<BasicBlockId> {
        self.basic_blocks[&block].next
    }

    pub fn append_block(&mut self, block: BasicBlockId) {
        self.basic_blocks.entry(block).or_insert(BasicBlockNode {
            prev: self.last_block,
            next: None,
            first_inst: None,
            last_inst: None,
        });

        if let Some(last_block) = self.last_block {
            self.basic_blocks.get_mut(&last_block).unwrap().next = Some(block);
            self.basic_blocks.get_mut(&block).unwrap().prev = Some(last_block);
        }

        self.last_block = Some(block);

        if self.first_block.is_none() {
            self.first_block = Some(block)
        }
    }

    pub fn append_inst(&mut self, inst: InstructionId, block: BasicBlockId) {
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

    pub fn remove_inst(&mut self, inst: InstructionId) -> Option<()> {
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

impl BasicBlockNode {
    pub fn last_inst(&self) -> &Option<InstructionId> {
        &self.last_inst
    }
}

impl<'a> Iterator for BasicBlockIter<'a> {
    type Item = BasicBlockId;

    fn next(&mut self) -> Option<Self::Item> {
        let cur = self.cur?;
        self.cur = self.layout.basic_blocks[&cur].next;
        Some(cur)
    }
}

impl<'a> Iterator for InstructionIter<'a> {
    type Item = InstructionId;

    fn next(&mut self) -> Option<Self::Item> {
        let cur = self.head?;
        if Some(cur) == self.layout.basic_blocks[&self.block].last_inst {
            self.head = None;
        } else {
            self.head = self.layout.instructions[&cur].next;
        }
        Some(cur)
    }
}

impl<'a> DoubleEndedIterator for InstructionIter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let cur = self.tail?;
        if self.head == self.tail {
            self.head = None;
            self.tail = None;
        } else {
            self.tail = self.layout.instructions[&cur].prev;
        }
        Some(cur)
    }
}
