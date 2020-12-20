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

    pub fn to_string(&self, data: &Data, types: &Types) -> String {
        match self.opcode {
            Opcode::Alloca => {
                format!("alloca ")
            }
            Opcode::Ret => {
                let args = self.operand.args();
                format!(
                    "ret {}",
                    if args.len() == 0 {
                        "void".to_string()
                    } else {
                        data.value_ref(args[0]).to_string(data, types)
                    }
                )
            }
        }
    }
}

impl Opcode {
    pub fn with_block(self, parent: BasicBlockId) -> Instruction {
        Instruction {
            opcode: self,
            operand: Operand::Invalid,
            dest: None,
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
}
