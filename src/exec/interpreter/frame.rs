use rustc_hash::FxHashMap;

use crate::{
    exec::generic_value::GenericValue,
    ir::{
        function::{instruction::InstructionId, Function},
        module::Module,
        value::{ConstantData, ConstantInt, Value, ValueId},
    },
};

pub struct StackFrame<'a> {
    pub module: &'a Module,
    pub func: &'a Function,
    val_map: FxHashMap<InstructionId, GenericValue>,
}

impl<'a> StackFrame<'a> {
    pub fn new(module: &'a Module, func: &'a Function) -> Self {
        Self {
            module,
            func,
            val_map: FxHashMap::default(),
        }
    }

    pub fn add_inst_val(&mut self, id: InstructionId, val: GenericValue) {
        self.val_map.insert(id, val);
    }

    pub fn get_inst_val(&self, id: InstructionId) -> Option<GenericValue> {
        self.val_map.get(&id).copied()
    }

    pub fn get_val(&self, id: ValueId) -> Option<GenericValue> {
        match self.func.data.value_ref(id) {
            Value::Instruction(id) => self.get_inst_val(*id),
            Value::Constant(ConstantData::Int(ConstantInt::Int32(i))) => {
                Some(GenericValue::Int32(*i))
            }
            _ => None,
        }
    }
}
