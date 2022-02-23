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
    pub fn undef() -> Self {
        Self::Constant(ConstantValue::Undef)
    }

    pub fn as_inst(&self) -> &InstructionId {
        match self {
            Self::Instruction(id) => id,
            _ => panic!(),
        }
    }

    pub fn display<'a>(&'a self, data: &'a Data, types: &'a Types) -> DisplayValue<'a> {
        DisplayValue(self, data, types, true, None)
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

pub struct DisplayValue<'a>(
    pub &'a Value,
    pub &'a Data,
    pub &'a Types,
    pub bool,                                                // show type
    pub Option<Box<dyn Fn(&'a Value) -> Option<Name> + 'a>>, // value name resolver
);

impl<'a> DisplayValue<'a> {
    pub fn show_type(mut self, show: bool) -> Self {
        self.3 = show;
        self
    }

    pub fn set_name_fn(mut self, f: Box<dyn Fn(&'a Value) -> Option<Name> + 'a>) -> Self {
        self.4 = Some(f);
        self
    }
}

impl std::fmt::Display for DisplayValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Value::Constant(c) if self.3 => {
                write!(f, "{} {}", self.2.to_string(c.ty()), c.to_string(self.2))
            }
            Value::Constant(c) => {
                write!(f, "{}", c.to_string(self.2))
            }
            Value::Instruction(id) => {
                let inst = self.1.inst_ref(*id);
                // TODO: Show type
                if let Some(Name::Name(dest)) = &inst.dest {
                    write!(f, "%{}", dest)
                } else if let Some(name) = self.4.as_ref().map(|f| f(self.0)).flatten() {
                    write!(f, "%{}", name)
                } else {
                    write!(f, "%I{}", id.index()) // TODO
                }
            }
            Value::Argument(n) if self.3 => {
                if let Some(Name::Name(name)) = n.name.as_ref() {
                    write!(f, "{} %{}", self.2.to_string(n.ty()), name)
                } else if let Some(name) = self.4.as_ref().map(|f| f(self.0)).flatten() {
                    write!(f, "{} %{}", self.2.to_string(n.ty()), name)
                } else {
                    write!(f, "{} %{}", self.2.to_string(n.ty()), n.nth)
                }
            }
            Value::Argument(n) => {
                if let Some(Name::Name(name)) = n.name.as_ref() {
                    write!(f, "%{}", name)
                } else if let Some(name) = self.4.as_ref().map(|f| f(self.0)).flatten() {
                    write!(f, "{} %{}", self.2.to_string(n.ty()), name)
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
