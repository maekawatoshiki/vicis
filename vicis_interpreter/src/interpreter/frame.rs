use rustc_hash::FxHashMap;

use super::Context;
use crate::generic_value::GenericValue;
use vicis_core::ir::{
    function::{instruction::InstructionId, Function},
    value::{ConstantData, ConstantExpr, ConstantInt, Value, ValueId},
};

pub struct StackFrame<'a> {
    pub ctx: &'a Context<'a>,
    pub func: &'a Function,
    val_map: FxHashMap<InstructionId, GenericValue>,
    args: Vec<GenericValue>,
}

impl<'a> StackFrame<'a> {
    pub fn new(ctx: &'a Context<'a>, func: &'a Function, args: Vec<GenericValue>) -> Self {
        Self {
            ctx,
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
            Value::Constant(ConstantData::Int(ConstantInt::Int64(i))) => {
                Some(GenericValue::Int64(*i))
            }
            Value::Constant(ConstantData::GlobalRef(name)) => {
                if let Some(f) = self
                    .ctx
                    .module
                    .find_function_by_name(name.to_string().unwrap())
                {
                    return Some(GenericValue::id(f));
                }
                if let Some(g) = self.ctx.globals.get(name) {
                    return Some(*g);
                }
                None
            }
            Value::Argument(i) => self.args.get(*i).copied(),
            Value::Constant(ConstantData::Expr(ConstantExpr::GetElementPtr { args, .. })) => {
                match args[0] {
                    ConstantData::GlobalRef(ref name) => self.ctx.globals.get(name).copied(),
                    _ => todo!(),
                }
            }
            _ => None,
        }
    }
}
