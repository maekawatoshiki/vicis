use crate::codegen::{
    function::Function,
    module::Module,
    target::x86_64::{
        instruction::{InstructionData, MemoryOperand},
        X86_64,
    },
};
use either::Either;
use std::fmt;

pub fn print(f: &mut fmt::Formatter<'_>, module: &Module<X86_64>) -> fmt::Result {
    writeln!(f, "  .text")?;
    writeln!(f, "  .intel_syntax noprefix")?;

    for (_, func) in &module.functions {
        print_function(f, func)?
    }

    Ok(())
}

pub fn print_function(f: &mut fmt::Formatter<'_>, function: &Function<X86_64>) -> fmt::Result {
    writeln!(f, "  .globl {}", function.name)?;
    writeln!(f, "{}:", function.name)?;

    for block in function.layout.block_iter() {
        for inst in function.layout.inst_iter(block) {
            let inst = function.data.inst_ref(inst);
            match &inst.data {
                InstructionData::PUSH64 { r: Either::Left(r) } => writeln!(f, "  push {:?}", r)?,
                InstructionData::POP64 { r: Either::Left(r) } => writeln!(f, "  pop {:?}", r)?,
                InstructionData::MOVri32 {
                    dst: Either::Left(dst),
                    src,
                } => writeln!(f, "  mov {:?}, {}", dst, src)?,
                InstructionData::MOVmi32 {
                    dst: MemoryOperand::Slot(slot),
                    src,
                } => writeln!(f, "  mov {:?}, {}", slot, src)?,
                InstructionData::RET => writeln!(f, "  ret")?,
                _ => todo!(),
            }
        }
    }

    Ok(())
}

impl fmt::Display for Module<X86_64> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        print(f, self)
    }
}
