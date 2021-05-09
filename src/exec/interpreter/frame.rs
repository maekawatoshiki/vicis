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
    args: Vec<GenericValue>,
}

impl<'a> StackFrame<'a> {
    pub fn new(module: &'a Module, func: &'a Function, args: Vec<GenericValue>) -> Self {
        Self {
            module,
            func,
            val_map: FxHashMap::default(),
            args,
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
            Value::Constant(ConstantData::GlobalRef(name)) => {
                if let Some(f) = self.module.find_function_by_name(name.to_string().unwrap()) {
                    return Some(GenericValue::id(f));
                }
                None
            }
            Value::Argument(i) => self.args.get(*i).copied(),
            _ => None,
        }
    }
}
