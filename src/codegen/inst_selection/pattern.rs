use crate::ir::{
    function::Data,
    instruction::Instruction,
    instruction::Opcode,
    value::{ConstantData, ConstantInt, Value},
};

pub trait Matcher<'a, X, Y> {
    fn matches(&self, data: &'a Data, x: &'a X) -> Option<&'a Y>;
}

impl<'a, F> Matcher<'a, Value, i32> for F
where
    F: Fn(&'a Data, &'a Value) -> Option<&'a i32>,
{
    fn matches(&self, data: &'a Data, x: &'a Value) -> Option<&'a i32> {
        self(data, x)
    }
}

pub mod ir {
    use super::*;

    pub fn ret_void<'a>() -> impl Fn(&'a Data, &'a Instruction) -> Option<&'a ()> {
        |_data: &Data, inst: &Instruction| {
            if inst.opcode != Opcode::Ret {
                return None;
            }
            if inst.operand.args().len() > 0 {
                return None;
            }
            Some(&())
        }
    }

    pub fn ret<'a, V, Y: 'a>(val: V) -> impl Fn(&'a Data, &'a Instruction) -> Option<&'a Y>
    where
        V: Matcher<'a, Value, Y>,
    {
        move |data: &Data, inst: &Instruction| {
            if inst.opcode != Opcode::Ret {
                return None;
            }
            let args = inst.operand.args();
            if args.len() == 0 {
                return None;
            }
            val.matches(data, data.value_ref(args[0]))
        }
    }

    pub fn any_i32<'a>() -> impl Fn(&'a Data, &'a Value) -> Option<&'a i32> {
        move |_data: &Data, val: &Value| match val {
            Value::Constant(ConstantData::Int(ConstantInt::Int32(i))) => return Some(i),
            _ => None,
        }
    }
}
