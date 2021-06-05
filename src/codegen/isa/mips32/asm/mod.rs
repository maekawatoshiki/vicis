use crate::codegen::{
    function::Function,
    isa::mips32::{
        instruction::{Opcode, Operand, OperandData},
        register::reg_to_str,
        MIPS32,
    },
    module::Module,
};
use std::fmt;

pub fn print(f: &mut fmt::Formatter<'_>, module: &Module<MIPS32>) -> fmt::Result {
    writeln!(f, "  .text")?;
    writeln!(f, "  .align 2")?;

    for gv in module.global_variables.values() {
        if let Some(init) = &gv.init {
            let arr = init.as_array();
            if !arr.is_string {
                continue;
            }
            let mut s = vec![];
            for elem in &arr.elems {
                s.push(*elem.as_int().as_i8() as u8)
            }
            let s = ::std::str::from_utf8(s.as_slice()).unwrap().to_string();
            let s = s.trim_end_matches('\x00'); // TODO
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
    function: &Function<MIPS32>,
    fn_idx: usize,
) -> fmt::Result {
    if function.is_prototype {
        return Ok(());
    }

    writeln!(f, "  .globl {}", function.name)?;
    writeln!(f, "{}:", function.name)?;

    for block in function.layout.block_iter() {
        writeln!(f, "$LBL{}_{}:", fn_idx, block.index())?;
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
                // if matches!(operand.data, OperandData::MemStart) {
                //     i += 1;
                //     // write!(f, "{} ptr ", mem_size(&inst.data.opcode))?;
                //     // write!(f, "{}", mem_op(&inst.data.operands[i..i + 5]))?;
                //     i += 5 - 1;
                //     todo!()
                // } else {
                write_operand(f, &operand.data, fn_idx)?;
                // }
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

impl fmt::Display for Module<MIPS32> {
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
                Self::JR => "jr",
                Self::ADDI => "addi",
                Self::Phi => "PHI",
            }
        )
    }
}

fn write_operand(f: &mut fmt::Formatter<'_>, op: &OperandData, fn_idx: usize) -> fmt::Result {
    match op {
        OperandData::Reg(r) => write!(f, "${}", reg_to_str(r)),
        OperandData::VReg(r) => write!(f, "%{}", r.0),
        OperandData::Slot(slot) => write!(f, "{:?}", slot),
        OperandData::Int32(i) => write!(f, "{}", i),
        OperandData::Block(block) => write!(f, "$LBL{}_{}", fn_idx, block.index()),
        OperandData::Label(name) => write!(f, "{}", name),
        OperandData::MemStart => Ok(()),
        OperandData::GlobalAddress(name) => write!(f, "offset {}", name),
        OperandData::None => write!(f, "none"),
    }
}
