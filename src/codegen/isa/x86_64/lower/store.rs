use super::get_or_generate_inst_output;
use crate::codegen::{
    function::instruction::Instruction as MachInstruction,
    isa::x86_64::{
        instruction::{InstructionData, Opcode, Operand as MOperand, OperandData},
        X86_64,
    },
    isa::TargetIsa,
    lower::{LoweringContext, LoweringError},
};
use crate::ir::{
    function::instruction::{InstructionId, Opcode as IrOpcode},
    types::{Type, TypeId},
    value::{ConstantData, ConstantInt, Value, ValueId},
};
use anyhow::Result;

pub fn lower_store(
    ctx: &mut LoweringContext<X86_64>,
    tys: &[TypeId],
    args: &[ValueId],
    _align: u32,
) -> Result<()> {
    let mut dst_slot = None;

    match ctx.ir_data.value_ref(args[1]) {
        Value::Instruction(id) => {
            // Maybe Alloca
            if let Some(slot_id) = ctx.inst_id_to_slot_id.get(id) {
                dst_slot = Some(*slot_id);
            } else {
                if ctx.ir_data.instructions[*id].opcode == IrOpcode::GetElementPtr {
                    return lower_store_gep(ctx, tys, args, _align, *id);
                }
            }
        }
        _ => return Err(LoweringError::Todo.into()),
    }

    let mut imm = None;
    let mut inst = None;

    match ctx.ir_data.value_ref(args[0]) {
        Value::Constant(ConstantData::Int(int)) => imm = Some(*int),
        Value::Instruction(id) => inst = Some(*id),
        _ => {}
    }

    match (dst_slot, inst, imm) {
        (Some(slot), Some(id), None) => {
            let inst = get_or_generate_inst_output(ctx, tys[0], id)?;
            ctx.inst_seq
                .append(&mut vec![MachInstruction::new(InstructionData {
                    opcode: Opcode::MOVmr32,
                    operands: vec![
                        MOperand::new(OperandData::MemStart),
                        MOperand::new(OperandData::Slot(slot)),
                        MOperand::new(OperandData::None),
                        MOperand::input(OperandData::None),
                        MOperand::input(OperandData::None),
                        MOperand::new(OperandData::None),
                        MOperand::input(inst.into()),
                    ],
                })]);
            return Ok(());
        }
        (Some(slot), None, Some(ConstantInt::Int32(imm))) => {
            ctx.inst_seq
                .append(&mut vec![MachInstruction::new(InstructionData {
                    opcode: Opcode::MOVmi32,
                    operands: vec![
                        MOperand::new(OperandData::MemStart),
                        MOperand::new(OperandData::Slot(slot)),
                        MOperand::new(OperandData::None),
                        MOperand::input(OperandData::None),
                        MOperand::input(OperandData::None),
                        MOperand::new(OperandData::None),
                        MOperand::input(imm.into()),
                    ],
                })]);
            return Ok(());
        }
        _ => return Err(LoweringError::Todo.into()),
    }
}

fn lower_store_gep(
    ctx: &mut LoweringContext<X86_64>,
    tys: &[TypeId],
    args: &[ValueId],
    _align: u32,
    gep_id: InstructionId,
) -> Result<()> {
    use {
        Constant as Const,
        ConstantData::Int,
        ConstantInt::{Int32, Int64},
        Value::Constant,
    };

    let mem;
    let gep = &ctx.ir_data.instructions[gep_id];
    let gep_args: Vec<&Value> = gep
        .operand
        .args()
        .into_iter()
        .map(|&arg| &ctx.ir_data.values[arg])
        .collect();

    match &gep_args[..] {
        [Value::Instruction(base_ptr), Const(Int(Int64(idx0))), Const(Int(Int64(idx1)))] => {
            let base_ptr = ctx.inst_id_to_slot_id[base_ptr];
            let base_ty = gep.operand.types()[0];
            let offset = idx0 * X86_64::type_size(ctx.types, base_ty) as i64
                + idx1
                    * X86_64::type_size(ctx.types, ctx.types.get_element(base_ty).unwrap()) as i64;
            // debug!(offset);

            mem = vec![
                MOperand::new(OperandData::MemStart),
                MOperand::new(OperandData::Slot(base_ptr)),
                MOperand::new(OperandData::Int32(offset as i32)),
                MOperand::input(OperandData::None),
                MOperand::input(OperandData::None),
                MOperand::new(OperandData::None),
            ];
        }
        [Value::Instruction(base_ptr), Const(Int(Int64(idx0))), Value::Instruction(idx1)] => {
            let base_ptr = ctx.inst_id_to_slot_id[base_ptr];

            let base_ty = gep.operand.types()[0];
            let offset = idx0 * X86_64::type_size(ctx.types, base_ty) as i64;
            // debug!(offset);

            let idx1_ty = gep.operand.types()[3];
            assert_eq!(*ctx.types.get(idx1_ty), Type::Int(64));
            let idx1 = get_or_generate_inst_output(ctx, idx1_ty, *idx1)?;

            assert!(X86_64::type_size(ctx.types, ctx.types.get_element(base_ty).unwrap()) == 4);

            mem = vec![
                MOperand::new(OperandData::MemStart),
                MOperand::new(OperandData::Slot(base_ptr)),
                MOperand::new(OperandData::Int32(offset as i32)),
                MOperand::input(OperandData::None),
                MOperand::input(OperandData::VReg(idx1)),
                MOperand::new(OperandData::Int32(X86_64::type_size(
                    ctx.types,
                    ctx.types.get_element(base_ty).unwrap(),
                ) as i32)),
            ];
        }
        _ => return Err(LoweringError::Todo.into()),
    }

    let src = args[0];
    let src_ty = tys[0];
    match ctx.ir_data.value_ref(src) {
        Const(Int(Int32(int))) => {
            ctx.inst_seq
                .append(&mut vec![MachInstruction::new(InstructionData {
                    opcode: Opcode::MOVmi32,
                    operands: mem
                        .into_iter()
                        .chain(vec![MOperand::input(int.into())].into_iter())
                        .collect(),
                })]);
        }
        Value::Instruction(id) => {
            let src = get_or_generate_inst_output(ctx, src_ty, *id)?;
            ctx.inst_seq
                .append(&mut vec![MachInstruction::new(InstructionData {
                    opcode: Opcode::MOVmr32,
                    operands: mem
                        .into_iter()
                        .chain(vec![MOperand::input(src.into())].into_iter())
                        .collect(),
                })]);
        }
        _ => return Err(LoweringError::Todo.into()),
    }

    Ok(())
}
