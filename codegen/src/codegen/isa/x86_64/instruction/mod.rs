use crate::codegen::{
    function::{
        basic_block::BasicBlockId,
        instruction::{Instruction, InstructionData as ID, InstructionId, InstructionInfo as II},
        slot::SlotId,
        Function,
    },
    isa::{x86_64::register::reg_to_str, TargetIsa},
    register::{Reg, VReg, VRegUsers},
};
use std::fmt;

pub struct InstructionInfo;

#[derive(Clone)]
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
    SUBri32,
    SUBrr32,
    MOVrr32,
    MOVrr64,
    MOVri32,
    MOVri64,
    MOVrm32,
    MOVmi32,
    MOVmr32,
    MOVSXDr64r32,
    MOVSXDr64m32,
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

#[derive(Clone)]
pub struct Operand {
    pub data: OperandData,
    pub input: bool,
    pub output: bool,
    pub implicit: bool,
}

#[derive(Clone)]
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

    fn store_vreg_to_slot<T: TargetIsa>(
        f: &Function<T>,
        vreg: VReg,
        slot: SlotId,
        block: BasicBlockId,
    ) -> Instruction<Self::Data> {
        let ty = f.data.vregs.type_for(vreg);
        assert!(ty.is_i32());
        Instruction::new(
            InstructionData {
                opcode: Opcode::MOVmr32,
                operands: vec![
                    Operand::new(OperandData::MemStart),
                    Operand::new(OperandData::Slot(slot)),
                    Operand::new(OperandData::None),
                    Operand::input(OperandData::None),
                    Operand::input(OperandData::None),
                    Operand::new(OperandData::None),
                    Operand::input(vreg.into()),
                ],
            },
            block,
        )
    }

    fn load_from_slot<T: TargetIsa>(
        f: &Function<T>,
        vreg: VReg,
        slot: SlotId,
        block: BasicBlockId,
    ) -> Instruction<Self::Data> {
        let ty = f.data.vregs.type_for(vreg);
        assert!(ty.is_i32());
        Instruction::new(
            InstructionData {
                opcode: Opcode::MOVrm32,
                operands: vec![
                    Operand::output(vreg.into()),
                    Operand::new(OperandData::MemStart),
                    Operand::new(OperandData::Slot(slot)),
                    Operand::new(OperandData::None),
                    Operand::input(OperandData::None),
                    Operand::input(OperandData::None),
                    Operand::new(OperandData::None),
                ],
            },
            block,
        )
    }
}

impl ID for InstructionData {
    fn input_vregs(&self) -> Vec<VReg> {
        let mut vrs = vec![];
        for operand in &self.operands {
            if let Operand {
                data: OperandData::VReg(vr),
                input: true,
                ..
            } = operand
            {
                vrs.push(*vr)
            }
        }
        vrs
    }

    fn output_vregs(&self) -> Vec<VReg> {
        let mut vrs = vec![];
        for operand in &self.operands {
            if let Operand {
                data: OperandData::VReg(vr),
                output: true,
                ..
            } = operand
            {
                vrs.push(*vr)
            }
        }
        vrs
    }

    fn all_vregs(&self) -> Vec<VReg> {
        let mut list = vec![];
        for operand in &self.operands {
            if let Operand {
                data: OperandData::VReg(r),
                ..
            } = operand
            {
                list.push(*r)
            }
        }
        list
    }

    fn input_regs(&self) -> Vec<Reg> {
        let mut rs = vec![];
        for operand in &self.operands {
            if let Operand {
                data: OperandData::Reg(r),
                input: true,
                ..
            } = operand
            {
                rs.push(*r)
            }
        }
        rs
    }

    fn output_regs(&self) -> Vec<Reg> {
        let mut rs = vec![];
        for operand in &self.operands {
            if let Operand {
                data: OperandData::Reg(r),
                output: true,
                ..
            } = operand
            {
                rs.push(*r)
            }
        }
        rs
    }

    fn all_regs(&self) -> Vec<Reg> {
        let mut list = vec![];
        for operand in &self.operands {
            if let Operand {
                data: OperandData::Reg(r),
                ..
            } = operand
            {
                list.push(*r)
            }
        }
        list
    }

    fn rewrite(&mut self, vreg: VReg, reg: Reg) {
        for operand in &mut self.operands {
            match operand.data {
                OperandData::VReg(vr) if vr == vreg => operand.data = OperandData::Reg(reg),
                _ => {}
            }
        }
    }

    fn replace_vreg(
        &mut self,
        self_id: InstructionId<Self>,
        users: &mut VRegUsers<Self>,
        from: VReg,
        to: VReg,
    ) {
        let u = users.remove_use(from, self_id).unwrap();
        users.add_use(to, self_id, u.read, u.write);
        for operand in &mut self.operands {
            match operand.data {
                OperandData::VReg(r) if r == from => operand.data = OperandData::VReg(to),
                _ => {}
            }
        }
    }

    fn is_copy(&self) -> bool {
        matches!(self.opcode, Opcode::MOVrr32 | Opcode::MOVrr64)
    }

    fn is_call(&self) -> bool {
        matches!(self.opcode, Opcode::CALL)
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

impl From<VReg> for OperandData {
    fn from(r: VReg) -> Self {
        OperandData::VReg(r)
    }
}

impl From<Reg> for OperandData {
    fn from(r: Reg) -> Self {
        OperandData::Reg(r)
    }
}

impl From<i32> for OperandData {
    fn from(i: i32) -> Self {
        OperandData::Int32(i)
    }
}

impl From<&i32> for OperandData {
    fn from(i: &i32) -> Self {
        OperandData::Int32(*i)
    }
}

impl fmt::Debug for InstructionData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} ", self.opcode)?;
        for (i, op) in self.operands.iter().enumerate() {
            write!(f, "{:?}", op)?;
            if i < self.operands.len() - 1 {
                write!(f, ", ")?
            }
        }
        Ok(())
    }
}

impl fmt::Debug for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut flags = vec![];
        // if self.input {
        //     flags.push("use")
        // }
        if self.output {
            flags.push("def")
        }
        if self.implicit {
            flags.push("imp")
        }
        write!(f, "{:?}", self.data)?;
        if !flags.is_empty() {
            write!(f, "<")?;
            for (i, flag) in flags.iter().enumerate() {
                write!(f, "{}", flag)?;
                if i < flags.len() - 1 {
                    write!(f, ", ")?
                }
            }
            write!(f, ">")?;
        }
        Ok(())
    }
}

impl fmt::Debug for OperandData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Reg(r) => write!(f, "{}", reg_to_str(r)),
            Self::VReg(vr) => write!(f, "%{}", vr.0),
            Self::Int32(i) => write!(f, "{}", i),
            Self::MemStart => write!(f, "$MemStart$"),
            Self::Slot(slot) => write!(f, "slot.{}", slot.index()),
            Self::Block(id) => write!(f, "block.{}", id.index()),
            Self::Label(name) => write!(f, "{}", name),
            Self::GlobalAddress(name) => write!(f, "{}", name),
            Self::None => write!(f, "none"),
        }
    }
}
