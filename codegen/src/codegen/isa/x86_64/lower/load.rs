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
use anyhow::Result;
use vicis_core::ir::{
    function::instruction::{InstructionId, Opcode as IrOpcode},
    types::Type,
    value::{ConstantInt, ConstantValue, Value, ValueId},
};

pub fn lower_load(
    ctx: &mut LoweringContext<X86_64>,
    id: InstructionId,
    tys: &[Type],
    addr: ValueId,
    _align: u32,
) -> Result<()> {
    let mut slot = None;

    // Very limited situation is supported now. TODO
    let sext = ctx.ir_data.only_one_user_of(id).filter(|&id| {
        let inst = ctx.ir_data.inst_ref(id);
        let types = inst.operand.types();
        inst.opcode == IrOpcode::Sext && types[0].is_i32() && types[1].is_i64()
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

        if sext.is_some() {
            let output = ctx.inst_id_to_vreg[&id];
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

        if src_ty.is_i32() {
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
    tys: &[Type],
    gep_id: InstructionId,
    _align: u32,
    sext: Option<InstructionId>,
) -> Result<()> {
    use {Constant as Const, ConstantInt::Int64, ConstantValue::Int, Value::Constant};

    let gep = &ctx.ir_data.instructions[gep_id];

    let gep_args: Vec<&Value> = gep
        .operand
        .args()
        .iter()
        .map(|&arg| &ctx.ir_data.values[arg])
        .collect();

    let mem = match &gep_args[..] {
        [Value::Instruction(base_ptr), Const(Int(Int64(idx0))), Const(Int(Int64(idx1)))] => {
            let base_ptr = ctx.inst_id_to_slot_id[base_ptr];
            let base_ty = gep.operand.types()[0];
            let offset = idx0 * ctx.isa.data_layout().get_size_of(ctx.types, base_ty) as i64
                + idx1
                    * ctx
                        .isa
                        .data_layout()
                        .get_size_of(ctx.types, ctx.types.get_element(base_ty).unwrap())
                        as i64;
            // debug!(offset);

            vec![
                MOperand::new(OperandData::MemStart),
                MOperand::new(OperandData::Slot(base_ptr)),
                MOperand::new(OperandData::Int32(offset as i32)),
                MOperand::input(OperandData::None),
                MOperand::input(OperandData::None),
                MOperand::new(OperandData::None),
            ]
        }
        [Value::Instruction(base_ptr), Const(Int(Int64(idx0))), Value::Instruction(idx1)] => {
            let base_ptr = ctx.inst_id_to_slot_id[base_ptr];

            let base_ty = gep.operand.types()[0];
            let offset = idx0 * ctx.isa.data_layout().get_size_of(ctx.types, base_ty) as i64;
            // debug!(offset);

            let idx1_ty = gep.operand.types()[3];
            assert!(idx1_ty.is_i64());
            let idx1 = get_or_generate_inst_output(ctx, idx1_ty, *idx1)?;

            assert!(
                ctx.isa
                    .data_layout()
                    .get_size_of(ctx.types, ctx.types.get_element(base_ty).unwrap())
                    == 4
            );

            vec![
                MOperand::new(OperandData::MemStart),
                MOperand::new(OperandData::Slot(base_ptr)),
                MOperand::new(OperandData::Int32(offset as i32)),
                MOperand::input(OperandData::None),
                MOperand::input(OperandData::VReg(idx1)),
                MOperand::new(OperandData::Int32(
                    ctx.isa
                        .data_layout()
                        .get_size_of(ctx.types, ctx.types.get_element(base_ty).unwrap())
                        as i32,
                )),
            ]
        }
        _ => return Err(LoweringError::Todo.into()),
    };

    let output = new_empty_inst_output(ctx, tys[0], sext.unwrap_or(id));

    let src_ty = tys[0];
    if src_ty.is_i32() {
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
    } else {
        return Err(LoweringError::Todo.into());
    }

    Ok(())
}
