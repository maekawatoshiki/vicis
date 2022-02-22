use super::{
    function::{data::Data, instruction::InstructionId},
    types::{Typed, Types},
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

    pub fn display<'a>(&'a self, data: &'a Data, types: &'a Types) -> DisplayValue<'a> {
        DisplayValue(self, data, types)
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

pub struct DisplayValue<'a>(pub &'a Value, pub &'a Data, pub &'a Types);

impl std::fmt::Display for DisplayValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Value::Constant(c) => write!(f, "{} {}", self.2.to_string(c.ty()), c.to_string(self.2)),
            Value::Instruction(id) => {
                let inst = self.1.inst_ref(*id);
                if let Some(dest) = &inst.dest {
                    write!(f, "%{}", dest)
                } else {
                    write!(f, "%I{}", id.index()) // TODO
                }
            }
            Value::Argument(n) => write!(f, "%A{}", n),
            Value::InlineAsm(InlineAsm {
                body,
                constraints,
                sideeffect,
            }) => write!(
                f,
                "{}asm {}\"{}\", \"{}\"",
                if *sideeffect { "sideeffect " } else { "" },
                constraints,
                body,
                body
            ),
        }
    }
}
