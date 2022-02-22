use rustc_hash::FxHashMap;

use super::Context;
use crate::{generic_value::GenericValue, interpreter::TypeSize};
use vicis_core::ir::{
    function::{instruction::InstructionId, Function},
    value::{ConstantExpr, ConstantInt, ConstantValue, Value, ValueId},
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

    pub fn get_inst_val(&self, id: InstructionId) -> Option<GenericValue> {
        self.val_map.get(&id).copied()
    }

    pub fn set_inst_val(&mut self, id: InstructionId, val: GenericValue) {
        self.val_map.insert(id, val);
    }

    pub fn get_val(&self, id: ValueId) -> Option<GenericValue> {
        match self.func.data.value_ref(id) {
            Value::Instruction(id) => self.get_inst_val(*id),
            Value::Constant(ConstantValue::Int(ConstantInt::Int1(i))) => {
                Some(GenericValue::Int1(*i))
            }
            Value::Constant(ConstantValue::Int(ConstantInt::Int8(i))) => {
                Some(GenericValue::Int8(*i))
            }
            Value::Constant(ConstantValue::Int(ConstantInt::Int32(i))) => {
                Some(GenericValue::Int32(*i))
            }
            Value::Constant(ConstantValue::Int(ConstantInt::Int64(i))) => {
                Some(GenericValue::Int64(*i))
            }
            Value::Constant(ConstantValue::GlobalRef(name)) => {
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
            Value::Argument(a) => self.args.get(a.nth).copied(),
            Value::Constant(ConstantValue::Expr(ConstantExpr::GetElementPtr { args, .. })) => {
                match args[0] {
                    ConstantValue::GlobalRef(ref name) => {
                        let n = match args[2] {
                            ConstantValue::Int(ConstantInt::Int32(n)) => n as i64,
                            ConstantValue::Int(ConstantInt::Int64(n)) => n,
                            _ => todo!(),
                        };
                        match self.ctx.globals.get(name).copied() {
                            Some(GenericValue::Ptr(v)) => {
                                let types = &self.ctx.module.types;
                                let ty = types
                                    .get_element(
                                        self.ctx.module.global_variables().get(name).unwrap().ty,
                                    )
                                    .unwrap();
                                let sz = types.size_of(ty) as i64;
                                Some(GenericValue::Ptr(((v as i64) + n * sz) as *mut u8))
                            }
                            Some(a) => Some(a),
                            None => None,
                        }
                    }
                    _ => todo!(),
                }
            }
            _ => None,
        }
    }
}
