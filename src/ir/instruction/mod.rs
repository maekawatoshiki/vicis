use super::{basic_block::BasicBlockId, module::name::Name, types::TypeId, value::ConstantData};
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
}

pub enum Operand {
    Alloca {
        ty: TypeId,
        num_elements: ConstantData,
        align: u32,
    },
}
