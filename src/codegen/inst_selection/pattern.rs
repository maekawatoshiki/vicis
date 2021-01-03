use crate::ir::{
    function::Data,
    instruction::Instruction,
    instruction::Opcode,
    value::{ConstantData, ConstantInt, Value},
};

pub trait Matcher<'a, X> {
    fn matches(&self, data: &'a Data, x: &'a X) -> Option<&'a X>;
}

impl<'a, F> Matcher<'a, Value> for F
where
    F: Fn(&'a Data, &'a Value) -> Option<&'a Value>,
{
    fn matches(&self, data: &'a Data, x: &'a Value) -> Option<&'a Value> {
        self(data, x)
    }
}

pub mod ir {
    use super::*;

    pub fn ret_void<'a>() -> impl Fn(&'a Data, &'a Instruction) -> Option<&'a Instruction> {
        |_data: &Data, inst: &Instruction| {
            if inst.opcode != Opcode::Ret {
                return None;
            }
            if inst.operand.args().len() > 0 {
                return None;
            }
            Some(inst)
        }
    }

    pub fn ret<'a, V>(val: V) -> impl Fn(&'a Data, &'a Instruction) -> Option<&'a Instruction>
    where
        V: Matcher<'a, Value>,
    {
        move |data: &Data, inst: &Instruction| {
            if inst.opcode != Opcode::Ret {
                return None;
            }
            let args = inst.operand.args();
            if args.len() == 0 {
                return None;
            }
            val.matches(data, data.value_ref(args[0]))?;
            Some(inst)
        }
    }

    pub fn any_i32<'a>() -> impl Fn(&'a Data, &'a Value) -> Option<&'a Value> {
        move |_data: &Data, val: &Value| {
            if matches!(
                val,
                Value::Constant(ConstantData::Int(ConstantInt::Int32(_)))
            ) {
                return Some(val);
            }
            None
        }
    }
}
