pub mod parser;

pub use parser::parse;

use super::{function::Data, instruction::InstructionId, module::name::Name, types::Types};
use id_arena::Id;

pub type ValueId = Id<Value>;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Instruction(InstructionId),
    Argument(usize),
    Constant(ConstantData),
    UnresolvedGlobalName(Name),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConstantData {
    Int(ConstantInt),
    // Expr(ConstantExprId, TypeId),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConstantInt {
    Int32(i32),
}

impl Value {
    pub fn to_string(&self, data: &Data, types: &Types) -> String {
        match self {
            Self::Constant(c) => c.to_string(data, types),
            Self::Instruction(id) => {
                format!("%I{}", id.index())
            }
            Self::Argument(n) => format!("%A{}", n),
            Self::UnresolvedGlobalName(n) => format!("@{:?}", n),
        }
    }
}

impl ConstantData {
    pub fn to_string(&self, _data: &Data, _types: &Types) -> String {
        match self {
            Self::Int(i) => i.to_string(),
        }
    }
}

impl ConstantInt {
    pub fn to_string(&self) -> String {
        match self {
            Self::Int32(i) => format!("{}", i),
        }
    }
}
