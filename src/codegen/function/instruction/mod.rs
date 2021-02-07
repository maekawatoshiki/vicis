use crate::codegen::register::{Reg, VReg};
use id_arena::Id;

pub type InstructionId<Data> = Id<Instruction<Data>>;

pub trait InstructionData: Clone {
    fn input_vregs(&self) -> Vec<VReg>;
    fn output_vregs(&self) -> Vec<VReg>;
    fn input_regs(&self) -> Vec<Reg>;
    fn output_regs(&self) -> Vec<Reg>;
    fn rewrite(&mut self, vreg: VReg, reg: Reg);
}

#[derive(Debug, Clone)]
pub struct Instruction<Data: InstructionData> {
    pub id: Option<InstructionId<Data>>,
    pub data: Data,
}

impl<Data: InstructionData> Instruction<Data> {
    pub fn new(data: Data) -> Self {
        Self { id: None, data }
    }
}
