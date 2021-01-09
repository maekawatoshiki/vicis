pub mod basic_block;
pub mod instruction;
pub mod parser;

pub use parser::parse;

use super::{
    module::{attributes::Attribute, name::Name, preemption_specifier::PreemptionSpecifier},
    types::{TypeId, Types},
    value::{Value, ValueId},
};
use basic_block::{BasicBlock, BasicBlockId};
use either::Either;
use id_arena::{Arena, Id};
use instruction::{Instruction, InstructionId};
use rustc_hash::FxHashMap;
use std::fmt;

/// `Attribute Group`s may be attached to `Function`s in the form of `#0`
pub type UnresolvedAttributeId = u32;

pub type FunctionId = Id<Function>;

pub struct Function {
    pub(crate) name: String,
    pub(crate) is_var_arg: bool,
    pub(crate) result_ty: TypeId,
    pub(crate) params: Vec<Parameter>,
    pub(crate) preemption_specifier: PreemptionSpecifier,
    pub(crate) attributes: Vec<Either<Attribute, UnresolvedAttributeId>>,
    pub data: Data,
    pub layout: Layout,
    pub types: Types,
    pub(crate) is_prototype: bool,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: Name,
    pub ty: TypeId,
    // pub attributes:
}

pub struct Data {
    pub values: Arena<Value>,
    pub instructions: Arena<Instruction>,
    pub basic_blocks: Arena<BasicBlock>,
}

pub struct Layout {
    basic_blocks: FxHashMap<BasicBlockId, BasicBlockNode>,
    instructions: FxHashMap<InstructionId, InstructionNode>,
    pub first_block: Option<BasicBlockId>,
    pub last_block: Option<BasicBlockId>,
}

pub struct BasicBlockNode {
    prev: Option<BasicBlockId>,
    next: Option<BasicBlockId>,
    first_inst: Option<InstructionId>,
    last_inst: Option<InstructionId>,
}

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

impl Data {
    pub fn new() -> Self {
        Self {
            values: Arena::new(),
            instructions: Arena::new(),
            basic_blocks: Arena::new(),
        }
    }

    pub fn create_block(&mut self) -> BasicBlockId {
        self.basic_blocks.alloc(BasicBlock::new())
    }

    pub fn create_inst(&mut self, mut inst: Instruction) -> InstructionId {
        let id = self.instructions.alloc_with_id(|id| {
            inst.id = Some(id);
            inst
        });
        self.set_inst_users(id);
        id
    }

    pub fn create_value(&mut self, inst: Value) -> ValueId {
        self.values.alloc(inst)
    }

    pub fn block_ref(&self, id: BasicBlockId) -> &BasicBlock {
        &self.basic_blocks[id]
    }

    // TODO: Is this the right way?
    pub fn block_ref_mut(&mut self, id: BasicBlockId) -> &mut BasicBlock {
        &mut self.basic_blocks[id]
    }

    pub fn inst_ref(&self, id: InstructionId) -> &Instruction {
        &self.instructions[id]
    }

    pub fn inst_ref_mut(&mut self, id: InstructionId) -> &mut Instruction {
        &mut self.instructions[id]
    }

    pub fn value_ref(&self, id: ValueId) -> &Value {
        &self.values[id]
    }

    // For `Instruction`s

    fn set_inst_users(&mut self, id: InstructionId) {
        let args = self.instructions[id]
            .operand
            .args()
            .into_iter()
            .filter_map(|&arg| match &self.values[arg] {
                Value::Instruction(id) => Some(*id),
                _ => None,
            })
            .collect::<Vec<InstructionId>>();
        for arg in args {
            self.instructions[arg].users.insert(id);
        }
    }

    fn remove_uses(&mut self, id: InstructionId) {
        let args = self.instructions[id]
            .operand
            .args()
            .into_iter()
            .filter_map(|&arg| match &self.values[arg] {
                Value::Instruction(id) => Some(*id),
                _ => None,
            })
            .collect::<Vec<InstructionId>>();
        for arg in args {
            self.instructions[arg].users.remove(&id);
        }
    }
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

    fn remove_inst(&mut self, inst: InstructionId) -> Option<()> {
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

impl Function {
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn is_var_arg(&self) -> bool {
        self.is_var_arg
    }

    pub fn remove_inst(&mut self, inst: InstructionId) -> Option<()> {
        self.data.remove_uses(inst);
        self.layout.remove_inst(inst)
    }
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_prototype {
            write!(f, "declare ")?
        } else {
            write!(f, "define ")?
        }
        write!(f, "{:?} ", self.preemption_specifier)?;
        write!(f, "{} ", self.types.to_string(self.result_ty))?;
        write!(f, "@{}(", self.name)?;
        for (i, param) in self.params.iter().enumerate() {
            write!(
                f,
                "{} %A{}{}",
                self.types.to_string(param.ty),
                i,
                if i == self.params.len() - 1 { "" } else { ", " }
            )?;
        }
        write!(f, ") ")?;
        for attr in &self.attributes {
            match attr {
                Either::Left(attr) => write!(f, "{:?} ", attr)?,
                Either::Right(id) => write!(f, "#{} ", id)?,
            }
        }

        if self.is_prototype {
            writeln!(f)?;
        } else {
            write!(f, "{{\n")?;
            for block_id in self.layout.block_iter() {
                writeln!(f, "B{:?}:", block_id.index())?;
                for inst_id in self.layout.inst_iter(block_id) {
                    let inst = self.data.inst_ref(inst_id);
                    writeln!(f, "    {}", inst.to_string(&self.data, &self.types))?;
                }
            }
            write!(f, "}}\n")?;
        }

        Ok(())
    }
}

impl Parameter {
    pub fn to_string(&self, types: &Types) -> String {
        format!("{} %{:?}", types.to_string(self.ty), self.name)
    }
}
