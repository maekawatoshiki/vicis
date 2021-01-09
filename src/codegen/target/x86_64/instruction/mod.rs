use crate::codegen::{
    function::{instruction::InstructionData as ID, slot::SlotId},
    register::{Reg, VReg},
};
// use crate::ir::instruction::InstructionId;

#[derive(Debug)]
pub struct InstructionData {
    pub opcode: Opcode,
    pub operands: Vec<Operand>,
}

#[derive(Debug)]
pub enum Opcode {
    PUSH64,
    POP64,
    ADDr64i32,
    ADDri32,
    ADDrr32,
    SUBr64i32,
    MOVrr32,
    MOVrr64,
    MOVri32,
    MOVrm32,
    MOVmi32,
    RET,
}

#[derive(Debug)]
pub struct Operand {
    pub data: OperandData,
    input: bool,
    output: bool,
}

#[derive(Debug)]
pub enum OperandData {
    Reg(Reg),
    VReg(VReg),
    Int32(i32),
    Mem(MemoryOperand),
}

#[derive(Debug)]
pub enum MemoryOperand {
    Slot(SlotId),
    ImmReg(i32, Reg),
}

impl InstructionData {
    pub fn mem_ops(&self) -> &[MemoryOperand] {
        for operand in &self.operands {
            match operand {
                Operand {
                    data: OperandData::Mem(mem),
                    ..
                } => return ::std::slice::from_ref(mem),
                _ => {}
            }
        }
        &[]
    }

    pub fn mem_ops_mut(&mut self) -> &mut [MemoryOperand] {
        for operand in &mut self.operands {
            match operand {
                Operand {
                    data: OperandData::Mem(mem),
                    ..
                } => return ::std::slice::from_mut(mem),
                _ => {}
            }
        }
        &mut []
    }
}

impl MemoryOperand {
    pub fn vregs(&self) -> Vec<VReg> {
        match self {
            Self::Slot(_) => vec![],
            Self::ImmReg(_, _) => vec![],
        }
    }

    pub fn regs(&self) -> Vec<Reg> {
        match self {
            Self::Slot(_) => vec![],
            Self::ImmReg(_, r) => vec![*r],
        }
    }
}

impl ID for InstructionData {
    fn input_vregs(&self) -> Vec<VReg> {
        let mut vrs = vec![];
        for operand in &self.operands {
            match operand {
                Operand {
                    data: OperandData::VReg(vr),
                    input: true,
                    ..
                } => vrs.push(*vr),
                _ => {}
            }
        }
        vrs
    }

    fn output_vregs(&self) -> Vec<VReg> {
        let mut vrs = vec![];
        for operand in &self.operands {
            match operand {
                Operand {
                    data: OperandData::VReg(vr),
                    output: true,
                    ..
                } => vrs.push(*vr),
                _ => {}
            }
        }
        vrs
    }

    fn input_regs(&self) -> Vec<Reg> {
        let mut rs = vec![];
        for operand in &self.operands {
            match operand {
                Operand {
                    data: OperandData::Reg(r),
                    input: true,
                    ..
                } => rs.push(*r),
                _ => {}
            }
        }
        rs
    }

    fn output_regs(&self) -> Vec<Reg> {
        let mut rs = vec![];
        for operand in &self.operands {
            match operand {
                Operand {
                    data: OperandData::Reg(r),
                    output: true,
                    ..
                } => rs.push(*r),
                _ => {}
            }
        }
        rs
    }

    fn rewrite(&mut self, vreg: VReg, reg: Reg) {
        for operand in &mut self.operands {
            match operand.data {
                OperandData::VReg(vr) if vr == vreg => operand.data = OperandData::Reg(reg),
                _ => {}
            }
        }
    }
}

impl Operand {
    pub fn new(data: OperandData) -> Self {
        Self {
            data,
            input: false,
            output: false,
        }
    }

    pub fn input(data: OperandData) -> Self {
        Self {
            data,
            input: true,
            output: false,
        }
    }

    pub fn output(data: OperandData) -> Self {
        Self {
            data,
            input: false,
            output: true,
        }
    }

    pub fn input_output(data: OperandData) -> Self {
        Self {
            data,
            input: true,
            output: true,
        }
    }
}

impl OperandData {
    pub fn as_reg(&self) -> &Reg {
        match self {
            Self::Reg(r) => r,
            _ => todo!(),
        }
    }
}
