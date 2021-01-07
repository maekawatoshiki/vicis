use super::register::VReg;
use id_arena::Id;

pub type InstructionId<Data> = Id<Instruction<Data>>;

pub trait InstructionData {
    fn input_vregs(&self) -> Vec<VReg>;
    fn output_vregs(&self) -> Vec<VReg>;
}

#[derive(Debug)]
pub struct Instruction<Data: InstructionData> {
    pub id: Option<InstructionId<Data>>,
    pub data: Data,
}
