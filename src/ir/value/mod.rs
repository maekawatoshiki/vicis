use super::{instruction::InstructionId, types::TypeId};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Instruction(InstructionId, TypeId),
    Argument(usize, TypeId),
    Constant(ConstantData),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConstantData {
    Int(ConstantInt),
    // Expr(ConstantExprId, TypeId),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConstantInt {
    Int32(i32),
}
