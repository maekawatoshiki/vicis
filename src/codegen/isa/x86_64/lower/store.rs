use super::get_or_generate_inst_output;
use crate::codegen::{
    function::instruction::Instruction as MachInstruction,
    isa::x86_64::{
        instruction::{InstructionData, Opcode, Operand as MOperand, OperandData},
        X86_64,
    },
    isa::Target,
    lower::LoweringContext,
};
use crate::ir::{
    function::instruction::{InstructionId, Opcode as IrOpcode},
    types::{Type, TypeId},
    value::{ConstantData, ConstantInt, Value, ValueId},
};

pub fn lower_store(
    ctx: &mut LoweringContext<X86_64>,
    tys: &[TypeId],
    args: &[ValueId],
    _align: u32,
) {
    let mut slot = None;

    match ctx.ir_data.value_ref(args[1]) {
        Value::Instruction(id) => {
            if let Some(slot_id) = ctx.inst_id_to_slot_id.get(id) {
                // Maybe Alloca
                slot = Some(*slot_id);
            } else {
                if ctx.ir_data.instructions[*id].opcode == IrOpcode::GetElementPtr {
                    return lower_store_gep(ctx, tys, args, _align, *id);
                }
            }
        }
        _ => todo!(),
    }

    let mut const_int = None;
    let mut inst = None;

    match ctx.ir_data.value_ref(args[0]) {
        Value::Constant(ConstantData::Int(int)) => const_int = Some(*int),
        Value::Instruction(id) => inst = Some(*id),
        _ => {}
    }

    match (slot, inst) {
        (Some(slot), Some(id)) => {
            let inst = get_or_generate_inst_output(ctx, tys[0], id);
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
            return;
        }
        _ => {}
    }

    match (slot, const_int) {
        (Some(slot), Some(ConstantInt::Int32(imm))) => {
            ctx.inst_seq.append(&mut vec![MachInstruction {
                id: None,
                data: InstructionData {
                    opcode: Opcode::MOVmi32,
                    operands: vec![
                        MOperand::new(OperandData::MemStart),
                        MOperand::output(OperandData::Slot(slot)),
                        MOperand::new(OperandData::None),
                        MOperand::input(OperandData::None),
                        MOperand::input(OperandData::None),
                        MOperand::new(OperandData::None),
                        MOperand::input(imm.into()),
                    ],
                },
            }]);
            return;
        }
        _ => todo!(),
    }
}

fn lower_store_gep(
    ctx: &mut LoweringContext<X86_64>,
    tys: &[TypeId],
    args: &[ValueId],
    _align: u32,
    gep_id: InstructionId,
) {
    let gep = &ctx.ir_data.instructions[gep_id];

    // The simplest pattern
    if let &[base_ptr, idx0, idx1] = gep.operand.args() {
        // let base_ty = gep.operand.types()[0];
        let base_ptr = ctx.inst_id_to_slot_id[ctx.ir_data.values[base_ptr].as_inst()];

        let idx0_ty = gep.operand.types()[2];
        assert_eq!(*ctx.types.get(idx0_ty), Type::Int(64));
        let idx0 = match ctx.ir_data.values[idx0] {
            Value::Constant(ConstantData::Int(ConstantInt::Int64(idx))) => idx,
            _ => todo!(),
        };

        let idx1_ty = gep.operand.types()[3];
        assert_eq!(*ctx.types.get(idx1_ty), Type::Int(64));
        let mut idx1_const = None;
        let mut idx1_vreg = None;
        match ctx.ir_data.values[idx1] {
            Value::Constant(ConstantData::Int(ConstantInt::Int64(idx))) => idx1_const = Some(idx),
            Value::Instruction(id) => {
                idx1_vreg = Some(get_or_generate_inst_output(ctx, idx1_ty, id))
            }
            _ => todo!(),
        };

        let mem_op = if let Some(idx1) = idx1_const {
            // idx0 * (size of base_ty) + idx1 * (size of inner of base_ty)
            let base_ty = gep.operand.types()[0];
            let offset = idx0 * X86_64::type_size(ctx.types, base_ty) as i64
                + idx1
                    * X86_64::type_size(ctx.types, ctx.types.get_element(base_ty).unwrap()) as i64;
            debug!(offset);

            vec![
                MOperand::new(OperandData::MemStart),
                MOperand::new(OperandData::Slot(base_ptr)),
                MOperand::new(OperandData::Int32(offset as i32)),
                MOperand::input(OperandData::None),
                MOperand::input(OperandData::None),
                MOperand::new(OperandData::None),
            ]
        } else if let Some(idx1) = idx1_vreg {
            let base_ty = gep.operand.types()[0];
            let offset = idx0 * X86_64::type_size(ctx.types, base_ty) as i64;
            debug!(offset);

            assert!(
                X86_64::type_size(ctx.types, ctx.types.get_element(base_ty).unwrap()) as i8 == 4
            );

            vec![
                MOperand::new(OperandData::MemStart),
                MOperand::new(OperandData::Slot(base_ptr)),
                MOperand::new(OperandData::Int32(offset as i32)),
                MOperand::input(OperandData::None),
                MOperand::input(OperandData::VReg(idx1)),
                MOperand::new(OperandData::Int32(X86_64::type_size(
                    ctx.types,
                    ctx.types.get_element(base_ty).unwrap(),
                ) as i32)),
            ]
        } else {
            panic!()
        };

        match ctx.ir_data.value_ref(args[0]) {
            Value::Constant(ConstantData::Int(ConstantInt::Int32(int))) => {
                ctx.inst_seq
                    .append(&mut vec![MachInstruction::new(InstructionData {
                        opcode: Opcode::MOVmi32,
                        operands: mem_op
                            .into_iter()
                            .chain(vec![MOperand::input(int.into())].into_iter())
                            .collect(),
                    })]);
            }
            Value::Instruction(id) => {
                let src = get_or_generate_inst_output(ctx, tys[0], *id);
                ctx.inst_seq
                    .append(&mut vec![MachInstruction::new(InstructionData {
                        opcode: Opcode::MOVmr32,
                        operands: mem_op
                            .into_iter()
                            .chain(vec![MOperand::input(src.into())].into_iter())
                            .collect(),
                    })]);
            }
            _ => todo!(),
        }

        return;
    }

    todo!()
}
