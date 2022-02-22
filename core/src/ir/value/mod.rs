use super::{
    function::{data::Data, instruction::InstructionId},
    types::Types,
};
use id_arena::Id;

mod consts;
pub use consts::*;

pub type ValueId = Id<Value>;

/// A value in LLVM IR.
///
/// The original LLVM Value class has information about its uses and users.
/// However, `Value` here does not have such information.
/// Instead, only for [`Instruction`](super::function::instruction::Instruction)s
/// we track uses & users. (See [`Data`](Data) for details.)
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Instruction(InstructionId),
    Argument(usize),
    Constant(ConstantData),
    InlineAsm(InlineAsm),
}

#[derive(Debug, Clone, PartialEq)]
pub struct InlineAsm {
    pub body: String,
    pub constraints: String,
    pub sideeffect: bool,
}

impl Value {
    pub fn undef() -> Self {
        Self::Constant(ConstantData::Undef)
    }

    pub fn as_inst(&self) -> &InstructionId {
        match self {
            Self::Instruction(id) => id,
            _ => panic!(),
        }
    }

    pub fn to_string(&self, _data: &Data, types: &Types) -> String {
        match self {
            Self::Constant(c) => c.to_string(types),
            Self::Instruction(id) => {
                format!("%I{}", id.index())
            }
            Self::Argument(n) => format!("%A{}", n),
            Self::InlineAsm(InlineAsm {
                body,
                constraints,
                sideeffect,
            }) => {
                format!(
                    "asm {}\"{}\", \"{}\"",
                    if *sideeffect { "sideeffect " } else { "" },
                    constraints,
                    body
                )
            }
        }
    }
}

impl From<i32> for Value {
    fn from(i: i32) -> Self {
        Self::Constant(ConstantInt::Int32(i).into())
    }
}

impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Self::Constant(ConstantInt::Int64(i).into())
    }
}
