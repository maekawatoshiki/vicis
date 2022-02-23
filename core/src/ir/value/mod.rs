use super::{
    function::{data::Data, instruction::InstructionId},
    module::name::Name,
    types::{Typed, Types},
};
use id_arena::Id;

mod consts;
pub use consts::*;

mod arg;
pub use arg::*;

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
    Argument(ArgumentValue),
    Constant(ConstantValue),
    InlineAsm(InlineAsm),
}

#[derive(Debug, Clone, PartialEq)]
pub struct InlineAsm {
    pub body: String,
    pub constraints: String,
    pub sideeffect: bool,
}

impl Value {
    pub fn as_inst(&self) -> Option<&InstructionId> {
        match self {
            Self::Instruction(id) => Some(id),
            _ => None,
        }
    }

    pub fn display<'a>(&'a self, data: &'a Data, types: &'a Types) -> DisplayValue<'a> {
        DisplayValue {
            val: self,
            data,
            types,
            display_type: true,
            display_as_operand: false,
            name_fn: None,
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

pub struct DisplayValue<'a> {
    pub val: &'a Value,
    pub data: &'a Data,
    pub types: &'a Types,
    pub display_type: bool,
    pub display_as_operand: bool,
    pub name_fn: Option<Box<dyn Fn(&'a Value) -> Option<Name> + 'a>>, // value name resolver
}

impl<'a> DisplayValue<'a> {
    pub fn display_type(mut self, x: bool) -> Self {
        self.display_type = x;
        self
    }

    pub fn display_as_operand(mut self, x: bool) -> Self {
        self.display_as_operand = x;
        self
    }

    pub fn set_name_fn(mut self, f: Box<dyn Fn(&'a Value) -> Option<Name> + 'a>) -> Self {
        self.name_fn = Some(f);
        self
    }
}

impl std::fmt::Display for DisplayValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.val {
            Value::Constant(c) if self.display_type => {
                write!(
                    f,
                    "{} {}",
                    self.types.to_string(c.ty()),
                    c.to_string(self.types)
                )
            }
            Value::Constant(c) => {
                write!(f, "{}", c.to_string(self.types))
            }
            Value::Instruction(id) if self.display_as_operand => {
                let inst = self.data.inst_ref(*id);
                if self.display_type {
                    write!(f, "{} ", self.types.to_string(inst.ty()))?;
                }
                // TODO: Show type
                if let Some(Name::Name(dest)) = &inst.dest {
                    write!(f, "%{}", dest)
                } else if let Some(name) = self.name_fn.as_ref().map(|f| f(self.val)).flatten() {
                    write!(f, "%{}", name)
                } else {
                    write!(f, "%I{}", id.index()) // TODO
                }
            }
            Value::Instruction(_) => {
                todo!()
            }
            Value::Argument(n) => {
                if self.display_type {
                    write!(f, "{} ", self.types.to_string(n.ty()))?;
                }
                if let Some(Name::Name(name)) = n.name.as_ref() {
                    write!(f, "%{}", name)
                } else if let Some(name) = self.name_fn.as_ref().map(|f| f(self.val)).flatten() {
                    write!(f, "{} %{}", self.types.to_string(n.ty()), name)
                } else {
                    write!(f, "%{}", n.nth)
                }
            }
            Value::InlineAsm(InlineAsm {
                body,
                constraints,
                sideeffect,
            }) => write!(
                f,
                "asm {}\"{}\", \"{}\"",
                if *sideeffect { "sideeffect " } else { "" },
                constraints,
                body
            ),
        }
    }
}
