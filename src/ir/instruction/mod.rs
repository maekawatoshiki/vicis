pub mod parser;

pub use parser::parse;

use super::{
    basic_block::BasicBlockId,
    module::name::Name,
    types::TypeId,
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
