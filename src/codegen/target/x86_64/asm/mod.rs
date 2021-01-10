use crate::codegen::{
    calling_conv::CallingConv,
    function::Function,
    module::Module,
    register::Reg,
    target::x86_64::{
        instruction::{MemoryOperand, Opcode, OperandData},
        register::RegClass,
        X86_64,
    },
};
use std::fmt;

pub fn print<CC: CallingConv<RegClass>>(
    f: &mut fmt::Formatter<'_>,
    module: &Module<X86_64<CC>>,
) -> fmt::Result {
    writeln!(f, "  .text")?;
    writeln!(f, "  .intel_syntax noprefix")?;

    for (_, func) in &module.functions {
        print_function(f, func)?
    }

    Ok(())
}

pub fn print_function<CC: CallingConv<RegClass>>(
    f: &mut fmt::Formatter<'_>,
    function: &Function<X86_64<CC>>,
) -> fmt::Result {
    writeln!(f, "  .globl {}", function.name)?;
    writeln!(f, "{}:", function.name)?;

    for block in function.layout.block_iter() {
        writeln!(f, ".LBL{}:", block.index())?;
        for inst in function.layout.inst_iter(block) {
            let inst = function.data.inst_ref(inst);
            write!(f, "  {} ", inst.data.opcode)?;
            for (i, operand) in inst.data.operands.iter().enumerate() {
                if let OperandData::Mem(_) = &operand.data {
                    write!(f, "{} ptr ", mem_size(&inst.data.opcode))?
                }
                write!(f, "{}", operand.data)?;
                if i < inst.data.operands.len() - 1 {
                    write!(f, ", ")?
                }
            }
            writeln!(f)?;
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

impl<CC: CallingConv<RegClass>> fmt::Display for Module<X86_64<CC>> {
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
                Self::SUBr64i32 => "sub",
                Self::MOVrr32 => "mov",
                Self::MOVrr64 => "mov",
                Self::MOVri32 => "mov",
                Self::MOVrm32 => "mov",
                Self::MOVmi32 => "mov",
                Self::JMP => "jmp",
                Self::RET => "ret",
            }
        )
    }
}

impl fmt::Display for OperandData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Reg(r) => write!(f, "{}", reg_to_str(r)),
            Self::VReg(r) => write!(f, "%{}", r.0),
            Self::Mem(mem) => write!(f, "{}", mem),
            Self::Int32(i) => write!(f, "{}", i),
            Self::Block(block) => write!(f, ".LBL{}", block.index()),
        }
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

fn mem_size(opcode: &Opcode) -> &'static str {
    match opcode {
        Opcode::MOVrm32 | Opcode::MOVmi32 => "dword",
        _ => todo!(),
    }
}
