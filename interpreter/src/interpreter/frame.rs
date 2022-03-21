use rustc_hash::FxHashMap;

use super::Context;
use crate::generic_value::GenericValue;
use std::ptr;
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
            Value::Argument(a) => self.args.get(a.nth).copied(),
            Value::Constant(konst) => self.get_val_from_const(konst),
            _ => None,
        }
    }

    fn get_val_from_const(&self, konst: &ConstantValue) -> Option<GenericValue> {
        match konst {
            ConstantValue::Null(ty) if ty.is_pointer(&self.ctx.module.types) => {
                Some(GenericValue::Ptr(ptr::null_mut()))
            }
            ConstantValue::Int(ConstantInt::Int1(i)) => Some(GenericValue::Int1(*i)),
            ConstantValue::Int(ConstantInt::Int8(i)) => Some(GenericValue::Int8(*i)),
            ConstantValue::Int(ConstantInt::Int32(i)) => Some(GenericValue::Int32(*i)),
            ConstantValue::Int(ConstantInt::Int64(i)) => Some(GenericValue::Int64(*i)),
            ConstantValue::GlobalRef(name, _) => {
                if let Some(f) = self
                    .ctx
                    .module
                    .find_function_by_name(name.to_string().unwrap())
                {
                    return Some(GenericValue::id(f));
                }
                self.ctx.globals.get(name).copied()
            }
            ConstantValue::Expr(ConstantExpr::GetElementPtr { args, tys, .. }) => match args[0] {
                ConstantValue::GlobalRef(ref name, _) => {
                    assert!(matches!(args[1], ConstantValue::Int(i) if i.is_zero()));
                    let n = match args[2] {
                        ConstantValue::Int(i) => i.cast_to_i64(),
                        _ => todo!(),
                    };
                    match self.ctx.globals.get(name).copied()? {
                        GenericValue::Ptr(v) => {
                            let types = &self.ctx.module.types;
                            if tys[0].is_struct(&self.ctx.module.types) {
                                assert!(n == 0); // TODO
                                Some(GenericValue::Ptr(v))
                            } else {
                                // Array
                                let ty = types.get_element(tys[0]).unwrap();
                                let sz = self.ctx.module.target().datalayout.get_size_of(types, ty)
                                    as i64;
                                Some(GenericValue::Ptr(((v as i64) + n * sz) as *mut u8))
                            }
                        }
                        x => Some(x),
                    }
                }
                _ => todo!(),
            },
            ConstantValue::Expr(ConstantExpr::Bitcast { arg, .. }) => self.get_val_from_const(arg),
            _ => todo!(),
        }
    }
}
