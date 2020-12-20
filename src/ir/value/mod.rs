pub mod parser;

pub use parser::parse;

use super::{instruction::InstructionId, types::TypeId};
use id_arena::Id;

pub type ValueId = Id<Value>;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Instruction(InstructionId, TypeId),
    Argument(usize, TypeId),
    Constant(ConstantData),
    UnresolvedGlobalIdentifier(String, TypeId),
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
