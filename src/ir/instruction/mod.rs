pub mod parser;

pub use parser::parse;

use super::{
    basic_block::BasicBlockId,
    function::Data,
    module::name::Name,
    types::TypeId,
    types::Types,
    value::{ConstantData, ValueId},
};
use id_arena::Id;

pub type InstructionId = Id<Instruction>;

pub struct Instruction {
    pub opcode: Opcode,
    pub operand: Operand,
    pub dest: Option<Name>,
    pub id: Option<InstructionId>,
    pub parent: BasicBlockId,
    // pub result_ty: Option<TypeIdjj
}

pub enum Opcode {
    Alloca,
    Ret,
}

pub enum Operand {
    Alloca {
        ty: TypeId,
        num_elements: ConstantData,
        align: u32,
    },
    Ret {
        val: Option<ValueId>,
    },
    Invalid,
}

impl Instruction {
    pub fn with_operand(mut self, operand: Operand) -> Self {
        self.operand = operand;
        self
    }

    pub fn with_dest(mut self, dest: Name) -> Self {
        self.dest = Some(dest);
        self
    }

    pub fn to_string(&self, data: &Data, types: &Types) -> String {
        match &self.operand {
            Operand::Alloca {
                ty,
                num_elements,
                align,
            } => {
                // TODO: %id_{index} or %{self.dest}
                format!(
                    "%id_{} = alloca {}, {}, align {}",
                    self.id.unwrap().index(),
                    types.to_string(*ty),
                    num_elements.to_string(data, types),
                    align
                )
            }
            Operand::Ret { val: None } => format!("ret void"),
            Operand::Ret { val: Some(val) } => {
                format!("ret {}", data.value_ref(*val).to_string(data, types))
            }
            Operand::Invalid => panic!(),
        }
    }
}

impl Opcode {
    pub fn with_block(self, parent: BasicBlockId) -> Instruction {
        Instruction {
            opcode: self,
            operand: Operand::Invalid,
            dest: None,
            id: None,
            parent,
        }
    }
}

impl Operand {
    pub fn args(&self) -> &[ValueId] {
        match self {
            Self::Alloca { .. } => &[],
            Self::Ret { val } if val.is_none() => &[],
            Self::Ret { val } => ::std::slice::from_ref(val.as_ref().unwrap()),
            Self::Invalid => &[],
        }
    }

    pub fn types(&self) -> &[TypeId] {
        match self {
            Self::Alloca { ty, .. } => ::std::slice::from_ref(ty),
            Self::Ret { .. } => &[],
            Self::Invalid => &[],
        }
    }
}
