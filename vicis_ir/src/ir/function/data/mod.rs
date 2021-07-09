use crate::ir::{
    function::{
        basic_block::{BasicBlock, BasicBlockId},
        instruction::{Instruction, InstructionId},
    },
    value::{Value, ValueId},
};
use id_arena::Arena;
use rustc_hash::{FxHashMap, FxHashSet};

pub struct Data {
    pub values: Arena<Value>,
    pub instructions: Arena<Instruction>,
    pub basic_blocks: Arena<BasicBlock>,
    pub users_map: FxHashMap<InstructionId, FxHashSet<InstructionId>>,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            values: Arena::new(),
            instructions: Arena::new(),
            basic_blocks: Arena::new(),
            users_map: FxHashMap::default(),
        }
    }
}

impl Data {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_block(&mut self) -> BasicBlockId {
        self.basic_blocks.alloc(BasicBlock::new())
    }

    pub fn create_inst(&mut self, mut inst: Instruction) -> InstructionId {
        let id = self.instructions.alloc_with_id(|id| {
            inst.id = Some(id);
            inst
        });
        self.users_map.insert(id, FxHashSet::default());
        self.validate_inst_uses(id);
        id
    }

    pub fn create_value(&mut self, inst: Value) -> ValueId {
        self.values.alloc(inst)
    }

    pub fn replace_inst(&mut self, from: InstructionId, to: Instruction) {
        self.instructions[from].replace(to);
        self.validate_inst_uses(from)
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

    pub fn value_ref_mut(&mut self, id: ValueId) -> &mut Value {
        &mut self.values[id]
    }

    pub fn users_of(&self, id: InstructionId) -> &FxHashSet<InstructionId> {
        &self.users_map[&id]
    }

    /// If an instruction with `id` has the only one user, return it.
    /// Otherwise, return None.
    pub fn only_one_user_of(&self, id: InstructionId) -> Option<InstructionId> {
        if self.users_of(id).len() != 1 {
            return None;
        }
        self.users_of(id).iter().next().copied()
    }

    pub fn replace_inst_arg(&mut self, inst_id: InstructionId, from: InstructionId, to: ValueId) {
        let inst = &mut self.instructions[inst_id];
        let mut replaced = false;

        // Replace `from` args with `to`
        for arg_id in inst.operand.args_mut() {
            let arg = &self.values[*arg_id];
            if matches!(arg, Value::Instruction(i) if i == &from) {
                *arg_id = to;
                replaced = true;
            }
            continue;
        }

        // Return if no args replaced
        if !replaced {
            return;
        }

        // Update users map

        if let Some(map) = self.users_map.get_mut(&from) {
            assert!(map.remove(&inst_id));
        }

        if let Value::Instruction(to) = self.values[to] {
            self.users_map
                .entry(to)
                .or_insert_with(FxHashSet::default)
                .insert(inst_id);
        }
    }

    pub fn replace_all_uses(&mut self, inst_id: InstructionId, to: ValueId) {
        for user_id in self.users_map[&inst_id].clone() {
            self.replace_inst_arg(user_id, inst_id, to);
        }
    }

    pub fn validate_inst_uses(&mut self, id: InstructionId) {
        let args = self.instructions[id]
            .operand
            .args()
            .iter()
            .filter_map(|&arg| match &self.values[arg] {
                Value::Instruction(id) => Some(*id),
                _ => None,
            })
            .collect::<Vec<InstructionId>>();
        for arg in args {
            self.users_map
                .entry(arg)
                .or_insert_with(FxHashSet::default)
                .insert(id);
        }
    }

    pub fn remove_uses(&mut self, id: InstructionId) {
        let args = self.instructions[id]
            .operand
            .args()
            .iter()
            .filter_map(|&arg| match &self.values[arg] {
                Value::Instruction(id) => Some(*id),
                _ => None,
            })
            .collect::<Vec<InstructionId>>();
        for arg in args {
            if let Some(users) = self.users_map.get_mut(&arg) {
                users.remove(&id);
            }
        }
    }
}
