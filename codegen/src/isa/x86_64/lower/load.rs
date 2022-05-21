use super::{get_inst_output, new_empty_inst_output};
use crate::{
    function::instruction::Instruction as MachInstruction,
    isa::x86_64::{
        instruction::{InstructionData, Opcode, Operand as MOperand, OperandData},
        lower::get_operand_for_val,
        X86_64,
    },
    isa::TargetIsa,
    lower::{LoweringContext, LoweringError},
};
use anyhow::Result;
use vicis_core::ir::{
    function::instruction::{InstructionId, Opcode as IrOpcode},
    types::{self, Type},
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
    let mut vreg = None;
    let mut gbl = None;

    // Very limited situation is supported now. TODO
    let sext = ctx.ir_data.only_one_user_of(id).filter(|&id| {
        let inst = ctx.ir_data.inst_ref(id);
        let types = inst.operand.types();
        inst.opcode == IrOpcode::Sext && types[0].is_i32() && types[1].is_i64()
    });

    match &ctx.ir_data.values[addr] {
        Value::Instruction(addr_id) => {
            if let Some(slot_id) = ctx.inst_id_to_slot_id.get(addr_id) {
                slot = Some(*slot_id);
            } else {
                let opcode = ctx.ir_data.instructions[*addr_id].opcode;
                if opcode == IrOpcode::GetElementPtr {
                    return lower_load_gep(ctx, id, tys, *addr_id, _align, sext);
                }
                vreg = Some(get_inst_output(ctx, tys[1], *addr_id)?);
            }
        }
        Value::Constant(ConstantValue::GlobalRef(name, _ty)) => {
            gbl = Some(name);
        }
        _ => return Err(LoweringError::Todo("Unsupported load pattern".into()).into()),
    }

    let mem;
    let src_ty = tys[0];

    if let Some(slot) = slot {
        mem = vec![
            MOperand::new(OperandData::MemStart),
            MOperand::new(OperandData::None),
            MOperand::new(OperandData::Slot(slot)),
            MOperand::new(OperandData::None),
            MOperand::input(OperandData::None),
            MOperand::input(OperandData::None),
            MOperand::new(OperandData::None),
        ];
    } else if let Some(gbl) = gbl {
        mem = vec![
            MOperand::new(OperandData::MemStart),
            MOperand::new(OperandData::Label(gbl.as_string().to_owned())),
            MOperand::new(OperandData::None),
            MOperand::new(OperandData::None),
            MOperand::input(OperandData::None),
            MOperand::input(OperandData::None),
            MOperand::new(OperandData::None),
        ]
    } else if let Some(vreg) = vreg {
        mem = vec![
            MOperand::new(OperandData::MemStart),
            MOperand::new(OperandData::None),
            MOperand::new(OperandData::None),
            MOperand::new(OperandData::None),
            MOperand::input(OperandData::None),
            MOperand::input(OperandData::VReg(vreg)),
            MOperand::new(OperandData::None),
        ]
    } else {
        return Err(LoweringError::Todo("Unsupported load pattern".into()).into());
    }

    if let Some(sext) = sext {
        let output = new_empty_inst_output(ctx, types::I64, sext);
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

    let sz = ctx.isa.data_layout().get_size_of(ctx.types, src_ty);
    let output = new_empty_inst_output(ctx, src_ty, id);
    let opcode = match sz {
        1 => Opcode::MOVrm8,
        4 => Opcode::MOVrm32,
        8 => Opcode::MOVrm64,
        _ => return Err(LoweringError::Todo("Unsupported load pattern".into()).into()),
    };

    ctx.inst_seq.push(MachInstruction::new(
        InstructionData {
            opcode,
            operands: vec![MOperand::output(output.into())]
                .into_iter()
                .chain(mem.into_iter())
                .collect(),
        },
        ctx.block_map[&ctx.cur_block],
    ));

    Ok(())
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
                MOperand::new(OperandData::None),
                MOperand::new(OperandData::Slot(base_ptr)),
                MOperand::new(OperandData::Int32(offset as i32)),
                MOperand::input(OperandData::None),
                MOperand::input(OperandData::None),
                MOperand::new(OperandData::None),
            ]
        }
        [Value::Instruction(base_ptr), Const(Int(Int64(idx0))), Value::Instruction(idx1)] => {
            let mut slot = None;
            let mut base = None;
            if let Some(p) = ctx.inst_id_to_slot_id.get(base_ptr) {
                slot = Some(*p);
            } else {
                base = Some(get_operand_for_val(
                    ctx,
                    gep.operand.types()[1],
                    gep.operand.args()[0],
                )?);
            }

            let base_ty = gep.operand.types()[0];
            let offset = idx0 * ctx.isa.data_layout().get_size_of(ctx.types, base_ty) as i64;
            // debug!(offset);

            let idx1_ty = gep.operand.types()[3];
            assert!(idx1_ty.is_i64());
            let idx1 = get_inst_output(ctx, idx1_ty, *idx1)?;

            assert!({
                let mul = ctx
                    .isa
                    .data_layout()
                    .get_size_of(ctx.types, ctx.types.get_element(base_ty).unwrap());
                mul == 1 || mul == 2 || mul == 4 || mul == 8
            });

            vec![
                MOperand::new(OperandData::MemStart),
                MOperand::new(OperandData::None),
                MOperand::new(slot.map_or(OperandData::None, |s| OperandData::Slot(s))),
                MOperand::new(OperandData::Int32(offset as i32)),
                MOperand::input(base.map_or(OperandData::None, |x| x)),
                MOperand::input(OperandData::VReg(idx1)),
                MOperand::new(OperandData::Int32(
                    ctx.isa
                        .data_layout()
                        .get_size_of(ctx.types, ctx.types.get_element(base_ty).unwrap())
                        as i32,
                )),
            ]
        }
        _ => return Err(LoweringError::Todo("Unsupported GEP pattern for load".into()).into()),
    };

    ctx.mark_as_merged(gep_id);
    if let Some(sext) = sext {
        ctx.mark_as_merged(sext);
    }

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
        return Err(LoweringError::Todo("Load result must be i32".into()).into());
    }

    Ok(())
}
