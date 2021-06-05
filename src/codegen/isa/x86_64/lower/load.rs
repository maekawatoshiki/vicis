use super::{get_or_generate_inst_output, new_empty_inst_output};
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

pub fn lower_load(
    ctx: &mut LoweringContext<X86_64>,
    id: InstructionId,
    tys: &[TypeId],
    addr: ValueId,
    _align: u32,
) -> Result<()> {
    let mut slot = None;

    // Very limited situation is supported now. TODO
    let sext = ctx.ir_data.only_one_user_of(id).filter(|&id| {
        let inst = ctx.ir_data.inst_ref(id);
        let types = inst.operand.types();
        inst.opcode == IrOpcode::Sext
            && types[0] == ctx.types.base().i32()
            && types[1] == ctx.types.base().i64()
    });

    if let Value::Instruction(addr_id) = &ctx.ir_data.values[addr] {
        if let Some(slot_id) = ctx.inst_id_to_slot_id.get(addr_id) {
            slot = Some(*slot_id);
        } else {
            let opcode = ctx.ir_data.instructions[*addr_id].opcode;
            if opcode == IrOpcode::GetElementPtr {
                return lower_load_gep(ctx, id, tys, *addr_id, _align, sext);
            }
        }
    } else {
        panic!()
    }

    if let Some(slot) = slot {
        let src_ty = tys[0];
        let mem = vec![
            MOperand::new(OperandData::MemStart),
            MOperand::new(OperandData::Slot(slot)),
            MOperand::new(OperandData::None),
            MOperand::input(OperandData::None),
            MOperand::input(OperandData::None),
            MOperand::new(OperandData::None),
        ];

        if let Some(u) = sext {
            // let output = new_empty_inst_output(ctx, ctx.types.base().i64(), u);
            let output = ctx.inst_id_to_vreg[&id];
            ctx.mark_as_merged(u);
            ctx.inst_seq.push(MachInstruction::new(
                InstructionData {
                    opcode: Opcode::MOVSXDr64m32,
                    operands: vec![MOperand::output(output.into())]
                        .into_iter()
                        .chain(mem.into_iter())
                        .collect(),
                },
                ctx.block_map[&ctx.cur_block],
            ));
            return Ok(());
        }

        if matches!(&*ctx.types.get(src_ty), Type::Int(32)) {
            // let output = new_empty_inst_output(ctx, src_ty, id);
            let output = ctx.inst_id_to_vreg[&id];
            ctx.inst_seq.push(MachInstruction::new(
                InstructionData {
                    opcode: Opcode::MOVrm32,
                    operands: vec![MOperand::output(output.into())]
                        .into_iter()
                        .chain(mem.into_iter())
                        .collect(),
                },
                ctx.block_map[&ctx.cur_block],
            ));
            return Ok(());
        }
    }

    Err(LoweringError::Todo.into())
}

fn lower_load_gep(
    ctx: &mut LoweringContext<X86_64>,
    id: InstructionId,
    tys: &[TypeId],
    gep_id: InstructionId,
    _align: u32,
    sext: Option<InstructionId>,
) -> Result<()> {
    use {Constant as Const, ConstantData::Int, ConstantInt::Int64, Value::Constant};

    let gep = &ctx.ir_data.instructions[gep_id];

    let gep_args: Vec<&Value> = gep
        .operand
        .args()
        .iter()
        .map(|&arg| &ctx.ir_data.values[arg])
        .collect();

    let mem;

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

    let output = new_empty_inst_output(ctx, tys[0], sext.unwrap_or(id));
    if let Some(x) = sext {
        ctx.mark_as_merged(x)
    }

    let src_ty = tys[0];
    match &*ctx.types.get(src_ty) {
        Type::Int(32) => {
            ctx.inst_seq.append(&mut vec![MachInstruction::new(
                InstructionData {
                    opcode: sext.map_or(Opcode::MOVrm32, |_| Opcode::MOVSXDr64m32),
                    operands: vec![MOperand::output(OperandData::VReg(output))]
                        .into_iter()
                        .chain(mem.into_iter())
                        .collect(),
                },
                ctx.block_map[&ctx.cur_block],
            )]);
        }
        _ => return Err(LoweringError::Todo.into()),
    }

    Ok(())
}
