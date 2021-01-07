use crate::codegen::{
    function::Function,
    module::Module,
    register::Reg,
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
                InstructionData::PUSH64 { r: Either::Left(r) } => {
                    writeln!(f, "  push {}", reg_to_str(r))?
                }
                InstructionData::POP64 { r: Either::Left(r) } => {
                    writeln!(f, "  pop {}", reg_to_str(r))?
                }
                InstructionData::ADDr64i32 {
                    r: Either::Left(r),
                    imm,
                } => writeln!(f, "  add {}, {}", reg_to_str(r), imm)?,
                InstructionData::SUBr64i32 {
                    r: Either::Left(r),
                    imm,
                } => writeln!(f, "  sub {}, {}", reg_to_str(r), imm)?,
                InstructionData::MOVrr32 {
                    dst: Either::Left(dst),
                    src: Either::Left(src),
                } => writeln!(f, "  mov {}, {}", reg_to_str(dst), reg_to_str(src))?,
                InstructionData::MOVrr64 {
                    dst: Either::Left(dst),
                    src: Either::Left(src),
                } => writeln!(f, "  mov {}, {}", reg_to_str(dst), reg_to_str(src))?,
                InstructionData::MOVri32 {
                    dst: Either::Left(dst),
                    src,
                } => writeln!(f, "  mov {}, {}", reg_to_str(dst), src)?,
                InstructionData::MOVmi32 { dst, src } => {
                    writeln!(f, "  mov dword ptr {}, {}", dst, src)?
                }
                InstructionData::MOVrm32 {
                    dst: Either::Left(dst),
                    src,
                } => writeln!(f, "  mov {}, dword ptr {}", reg_to_str(dst), src)?,
                InstructionData::RET => writeln!(f, "  ret")?,
                _ => todo!(),
            }
        }
    }

    Ok(())
}

impl fmt::Display for MemoryOperand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ImmReg(imm, reg) => {
                write!(
                    f,
                    "[{}{}{}]",
                    reg_to_str(reg),
                    if *imm < 0 { "" } else { "+" },
                    *imm
                )
            }
            Self::Slot(_) => panic!(),
        }
    }
}

impl fmt::Display for Module<X86_64> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        print(f, self)
    }
}

fn reg_to_str(r: &Reg) -> &'static str {
    let gr32 = [
        "eax", "ecx", "edx", "ebx", "esp", "ebp", "esi", "edi", "r8", "r9d", "r10d", "r11d",
        "r12d", "r13d", "r14d", "r15d",
    ];
    let gr64 = [
        "rax", "rcx", "rdx", "rbx", "rsp", "rbp", "rsi", "rdi", "r8", "r9", "r10", "r11", "r12",
        "r13", "r14", "r15",
    ];
    match r {
        Reg(0, i) => gr32[*i as usize],
        Reg(1, i) => gr64[*i as usize],
        e => todo!("{:?}", e),
    }
}
