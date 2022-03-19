use crate::codegen::{
    function::Function,
    isa::x86_64::{
        instruction::{Opcode, Operand, OperandData},
        register::reg_to_str,
        X86_64,
    },
    module::Module,
};
use std::fmt;

pub fn print(f: &mut fmt::Formatter<'_>, module: &Module<X86_64>) -> fmt::Result {
    writeln!(f, "  .text")?;
    writeln!(f, "  .intel_syntax noprefix")?;

    for gv in module.ir.global_variables().values() {
        if let Some(init) = &gv.init {
            let arr = init.as_array();
            if !arr.is_string {
                continue;
            }
            let mut s = vec![];
            for elem in &arr.elems {
                s.push(*elem.as_int().as_i8() as u8)
            }
            let s: Vec<u8> = s
                .into_iter()
                .flat_map(::std::ascii::escape_default)
                .collect();
            let s = ::std::str::from_utf8(s.as_slice()).unwrap().to_string();
            let s = s.trim_end_matches("\\x00"); // TODO
            debug!(&s);
            writeln!(f, "{}:", gv.name.as_string())?;
            writeln!(f, "  .string \"{}\"", s)?;
        }
    }

    for (i, (_, func)) in module.functions.iter().enumerate() {
        print_function(f, func, i)?
    }

    Ok(())
}

pub fn print_function(
    f: &mut fmt::Formatter<'_>,
    function: &Function<X86_64>,
    fn_idx: usize,
) -> fmt::Result {
    if function.is_declaration {
        return Ok(());
    }

    writeln!(f, "  .globl {}", function.ir.name())?;
    writeln!(f, "{}:", function.ir.name())?;

    for block in function.layout.block_iter() {
        writeln!(f, ".LBL{}_{}:", fn_idx, block.index())?;
        for inst in function.layout.inst_iter(block) {
            let inst = function.data.inst_ref(inst);
            write!(f, "  {} ", inst.data.opcode)?;
            let mut i = 0;
            while i < inst.data.operands.len() {
                let operand = &inst.data.operands[i];
                if operand.implicit {
                    i += 1;
                    continue;
                }
                if matches!(operand.data, OperandData::MemStart) {
                    i += 1;
                    write!(f, "{} ptr ", mem_size(&inst.data.opcode))?;
                    write!(f, "{}", mem_op(&inst.data.operands[i..i + 5]))?;
                    i += 5 - 1;
                } else {
                    write_operand(f, &operand.data, fn_idx)?;
                }
                if i < inst.data.operands.len() - 1 {
                    write!(f, ", ")?
                }
                i += 1;
            }
            writeln!(f)?;
        }
    }

    Ok(())
}

impl fmt::Display for Module<'_, X86_64> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        print(f, self)
    }
}

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::PUSH64 => "push",
                Self::POP64 => "pop",
                Self::ADDr64i32 => "add",
                Self::ADDri32 => "add",
                Self::ADDrr32 => "add",
                Self::SUBri32 | Self::SUBrr32 | Self::SUBr64i32 => "sub",
                Self::MOVrr32 => "mov",
                Self::MOVrr64 => "mov",
                Self::MOVri32 => "mov",
                Self::MOVrm32 => "mov",
                Self::MOVmi32 => "mov",
                Self::MOVmr32 => "mov",
                Self::MOVSXDr64r32 | Self::MOVSXDr64m32 => "movsxd",
                Self::CMPri32 => "cmp",
                Self::JMP => "jmp",
                Self::JE => "je",
                Self::JNE => "jne",
                Self::JLE => "jle",
                Self::JL => "jl",
                Self::JGE => "jge",
                Self::JG => "jg",
                Self::CALL => "call",
                Self::RET => "ret",
                Self::Phi => "PHI",
            }
        )
    }
}

fn write_operand(f: &mut fmt::Formatter<'_>, op: &OperandData, fn_idx: usize) -> fmt::Result {
    match op {
        OperandData::Reg(r) => write!(f, "{}", reg_to_str(r)),
        OperandData::VReg(r) => write!(f, "%{}", r.0),
        OperandData::Slot(slot) => write!(f, "{:?}", slot),
        OperandData::Int32(i) => write!(f, "{}", i),
        OperandData::Block(block) => write!(f, ".LBL{}_{}", fn_idx, block.index()),
        OperandData::Label(name) => write!(f, "{}", name),
        OperandData::MemStart => Ok(()),
        OperandData::GlobalAddress(name) => write!(f, "offset {}", name),
        OperandData::None => write!(f, "none"),
    }
}

// impl fmt::Display for OperandData {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             Self::Reg(r) => write!(f, "{}", reg_to_str(r)),
//             Self::VReg(r) => write!(f, "%{}", r.0),
//             // Self::Mem(mem) => write!(f, "{}", mem),
//             Self::Slot(slot) => write!(f, "{:?}", slot),
//             Self::Int32(i) => write!(f, "{}", i),
//             Self::Block(block) => write!(f, ".LBL{}", block.index()),
//             Self::Label(name) => write!(f, "{}", name),
//             Self::MemStart => Ok(()),
//             Self::GlobalAddress(name) => write!(f, "offset {}", name),
//             Self::None => write!(f, "none"),
//         }
//     }
// }

fn mem_size(opcode: &Opcode) -> &'static str {
    match opcode {
        Opcode::MOVrm32 | Opcode::MOVmi32 | Opcode::MOVmr32 | Opcode::MOVSXDr64m32 => "dword",
        _ => todo!(),
    }
}

fn mem_op(args: &[Operand]) -> String {
    assert!(matches!(&args[0].data, &OperandData::None)); // assure slot is eliminated
    match (&args[1].data, &args[2].data, &args[3].data, &args[4].data) {
        (OperandData::Int32(imm), OperandData::Reg(reg), OperandData::None, OperandData::None) => {
            format!(
                "[{}{}{}]",
                reg_to_str(reg),
                if *imm < 0 { "" } else { "+" },
                *imm
            )
        }
        (
            OperandData::Int32(imm),
            OperandData::Reg(reg1),
            OperandData::Reg(reg2),
            OperandData::Int32(shift),
        ) => {
            format!(
                "[{}{}{}+{}*{}]",
                reg_to_str(reg1),
                if *imm < 0 { "" } else { "+" },
                *imm,
                reg_to_str(reg2),
                shift
            )
        }
        _ => todo!(),
    }
}
