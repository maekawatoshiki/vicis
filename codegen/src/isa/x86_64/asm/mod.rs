use vicis_core::ir::{
    module::{linkage::Linkage, name::Name},
    types::Typed,
    value::{ConstantInt, ConstantStruct, ConstantValue},
};

use crate::{
    function::Function,
    isa::{
        x86_64::{
            instruction::{Opcode, Operand, OperandData},
            register::reg_to_str,
            X86_64,
        },
        TargetIsa,
    },
    module::{DisplayAsm, Module},
};
use std::{fmt, str};

impl fmt::Display for DisplayAsm<'_, X86_64> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        print(f, self.0)
    }
}

pub fn print(f: &mut fmt::Formatter<'_>, module: &Module<X86_64>) -> fmt::Result {
    writeln!(f, "  .text")?;
    writeln!(f, "  .intel_syntax noprefix")?;

    for (i, (_, func)) in module.functions.iter().enumerate() {
        print_function(f, func, i)?
    }

    let mut ctor = None;

    for gv in module.ir.global_variables().values() {
        let is_extern = matches!(
            gv.linkage,
            Some(Linkage::External) | Some(Linkage::ExternalWeak)
        );
        if is_extern {
            continue;
        }

        if gv.init.is_none() {
            continue;
        }

        let init = gv.init.as_ref().unwrap();
        match init {
            ConstantValue::Array(arr) => {
                if matches!(gv.name, Name::Name(ref name) if name == "llvm.global_ctors") {
                    // TODO
                    if let ConstantValue::Struct(ConstantStruct { elems, .. }) = &arr.elems[0] {
                        if let ConstantValue::GlobalRef(name, _) = &elems[1] {
                            ctor = module.ir.find_function_by_name(name.as_string());
                        }
                    }
                    continue;
                }

                if !arr.is_string {
                    continue;
                }

                let mut s = vec![];
                for elem in &arr.elems {
                    s.push(*elem.as_int().unwrap().as_i8() as u8)
                }
                let s = str::from_utf8(
                    s.into_iter()
                        .flat_map(::std::ascii::escape_default)
                        .collect::<Vec<u8>>()
                        .as_slice(),
                )
                .unwrap()
                .to_string();
                let s = s.trim_end_matches("\\x00"); // TODO
                debug!(&s);
                writeln!(f, "{}:", gv.name.as_string())?;
                writeln!(f, "  .string \"{}\"", s)?;
            }
            ConstantValue::AggregateZero(ty) => {
                let size = module.isa.data_layout().get_size_of(&module.types, *ty);
                let align = module.isa.data_layout().get_align_of(&module.types, *ty);
                writeln!(f, "  .comm {},{},{}", gv.name.as_string(), size, align)?;
            }
            ConstantValue::Int(i) => {
                if !gv.linkage.map_or(false, |l| l.is_internal()) {
                    writeln!(f, "  .globl {}", gv.name.as_string())?;
                }
                let size = module.isa.data_layout().get_size_of(&module.types, i.ty());
                writeln!(f, "{}:", gv.name.as_string())?;
                writeln!(
                    f,
                    "  .{sz} {i}",
                    sz = match i {
                        ConstantInt::Int1(_) => "byte",
                        ConstantInt::Int8(_) => "byte",
                        ConstantInt::Int32(_) => "long",
                        ConstantInt::Int64(_) => "quad",
                    }
                )?;
                writeln!(f, "  .size {}, {}", gv.name.as_string(), size)?;
            }
            e => todo!("Unsupported initializer: {:?}", e),
        }
    }

    if let Some(ctor) = ctor {
        let ctor = &module.ir.functions()[ctor];
        writeln!(f, "  .section .init_array")?;
        writeln!(f, "  .quad {}", ctor.name())?;
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

    if let Some(name) = &function.ir.section {
        writeln!(f, "  .section {}", name)?;
    } else {
        writeln!(f, "  .text")?;
    }
    if !function.ir.linkage.is_internal() {
        writeln!(f, "  .globl {}", function.ir.name())?;
    }
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
                    let sz = mem_size(&inst.data.opcode);
                    write!(f, "{}", sz)?;
                    if !sz.is_empty() {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", mem_op(&inst.data.operands[i..i + 6]))?;
                    i += 6 - 1;
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

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::PUSH64 => "push",
                Self::POP64 => "pop",
                Self::ADDr64i32 | Self::ADDri32 | Self::ADDrr32 => "add",
                Self::SUBri32 | Self::SUBrr32 | Self::SUBr64i32 => "sub",
                Self::IMULrr32 => "imul",
                Self::MOVrm8
                | Self::MOVmr8
                | Self::MOVmi8
                | Self::MOVrr32
                | Self::MOVrr64
                | Self::MOVri32
                | Self::MOVri64
                | Self::MOVrm64
                | Self::MOVmr64
                | Self::MOVrm32
                | Self::MOVmi32
                | Self::MOVmi64
                | Self::MOVmr32 => "mov",
                Self::LEArm64 => "lea",
                Self::MOVSXDr64r32 | Self::MOVSXDr64m32 => "movsxd",
                Self::CMPri8 | Self::CMPri32 | Self::CMPrr32 => "cmp",
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
        OperandData::Int8(i) => write!(f, "{}", i),
        OperandData::Int32(i) => write!(f, "{}", i),
        OperandData::Int64(i) => write!(f, "{}", i),
        OperandData::Block(block) => write!(f, ".LBL{}_{}", fn_idx, block.index()),
        OperandData::Label(name) => write!(f, "{}", name),
        OperandData::MemStart => Ok(()),
        OperandData::GlobalAddress(name) => write!(f, "offset {}", name),
        OperandData::None => write!(f, "none"),
    }
}

fn mem_size(opcode: &Opcode) -> &'static str {
    match opcode {
        Opcode::MOVrm8 | Opcode::MOVmr8 | Opcode::MOVmi8 => "byte ptr",
        Opcode::MOVrm32 | Opcode::MOVmi32 | Opcode::MOVmr32 | Opcode::MOVSXDr64m32 => "dword ptr",
        Opcode::MOVmi64 | Opcode::MOVrm64 | Opcode::MOVmr64 => "qword ptr",
        Opcode::LEArm64 => "",
        e => todo!("{:?}", e),
    }
}

fn mem_op(args: &[Operand]) -> String {
    assert!(matches!(&args[1].data, &OperandData::None)); // assure slot is eliminated
    match (
        &args[0].data,
        &args[2].data,
        &args[3].data,
        &args[4].data,
        &args[5].data,
    ) {
        (
            OperandData::Label(name),
            OperandData::None,
            OperandData::None,
            OperandData::None,
            OperandData::None,
        ) => {
            format!("[{}]", name)
        }
        (
            OperandData::None,
            OperandData::Int32(imm),
            OperandData::Reg(reg),
            OperandData::None,
            OperandData::None,
        ) => {
            format!(
                "[{}{}{}]",
                reg_to_str(reg),
                if *imm < 0 { "" } else { "+" },
                *imm
            )
        }
        (
            OperandData::None,
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
        (
            OperandData::None,
            OperandData::None,
            OperandData::None,
            OperandData::Reg(reg2),
            OperandData::None,
        ) => {
            format!("[{}]", reg_to_str(reg2),)
        }
        _ => todo!(),
    }
}
