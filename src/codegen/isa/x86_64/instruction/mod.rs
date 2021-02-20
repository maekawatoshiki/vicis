use crate::codegen::{
    function::{
        basic_block::BasicBlockId,
        instruction::{InstructionData as ID, InstructionInfo as II},
        slot::SlotId,
    },
    register::{Reg, VReg},
};
// use crate::ir::instruction::InstructionId;

pub struct InstructionInfo;

#[derive(Debug, Clone)]
pub struct InstructionData {
    pub opcode: Opcode,
    pub operands: Vec<Operand>,
}

#[derive(Debug, Copy, Clone)]
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
    MOVmr32,
    MOVSXDr64r32,
    CMPri32,
    JMP,
    JE,
    JNE,
    JLE,
    JL,
    JGE,
    JG,
    CALL,
    RET,

    // TODO
    Phi,
}

#[derive(Debug, Clone)]
pub struct Operand {
    pub data: OperandData,
    pub input: bool,
    pub output: bool,
    pub implicit: bool,
}

#[derive(Debug, Clone)]
pub enum OperandData {
    Reg(Reg),
    VReg(VReg),
    Int32(i32),
    MemStart, // followed by: Slot, Imm, Reg(basically rbp), Reg, Shift
    Slot(SlotId),
    Block(BasicBlockId),
    Label(String),
    GlobalAddress(String),
    None,
}

impl II for InstructionInfo {
    type Data = InstructionData;
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

    fn is_copy(&self) -> bool {
        matches!(self.opcode, Opcode::MOVrr32 | Opcode::MOVrr64)
    }
}

impl Operand {
    pub fn new(data: OperandData) -> Self {
        Self {
            data,
            input: false,
            output: false,
            implicit: false,
        }
    }

    pub fn input(data: OperandData) -> Self {
        Self {
            data,
            input: true,
            output: false,
            implicit: false,
        }
    }

    pub fn output(data: OperandData) -> Self {
        Self {
            data,
            input: false,
            output: true,
            implicit: false,
        }
    }

    pub fn implicit_output(data: OperandData) -> Self {
        Self {
            data,
            input: false,
            output: true,
            implicit: true,
        }
    }

    pub fn input_output(data: OperandData) -> Self {
        Self {
            data,
            input: true,
            output: true,
            implicit: false,
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

    pub fn as_vreg(&self) -> &VReg {
        match self {
            Self::VReg(r) => r,
            _ => todo!(),
        }
    }

    pub fn as_block(&self) -> &BasicBlockId {
        match self {
            Self::Block(b) => b,
            _ => todo!(),
        }
    }
}

impl Into<OperandData> for VReg {
    fn into(self) -> OperandData {
        OperandData::VReg(self)
    }
}

impl Into<OperandData> for Reg {
    fn into(self) -> OperandData {
        OperandData::Reg(self)
    }
}

impl Into<OperandData> for i32 {
    fn into(self) -> OperandData {
        OperandData::Int32(self)
    }
}

impl Into<OperandData> for &i32 {
    fn into(self) -> OperandData {
        OperandData::Int32(*self)
    }
}
