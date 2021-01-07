use crate::codegen::{
    instruction::Instruction as MachInstruction,
    lower::pattern::{Lower as LowerTrait, LoweringContext},
    // register::VRegData,
    target::x86_64::{
        instruction::{InstructionData, MemoryOperand},
        register::GR32,
        X86_64,
    },
};
use crate::ir::{
    instruction::{Instruction as IrInstruction, InstructionId, Operand},
    types::{Type, TypeId},
    value::{ConstantData, ConstantInt, Value, ValueId},
};
use either::Either;

#[derive(Clone, Copy)]
pub struct Lower {}

impl Lower {
    pub fn new() -> Self {
        Lower {}
    }
}

impl LowerTrait<X86_64> for Lower {
    fn lower(&self, ctx: &mut LoweringContext<X86_64>, inst: &IrInstruction) {
        lower(ctx, inst)
    }
}

fn lower(ctx: &mut LoweringContext<X86_64>, inst: &IrInstruction) {
    match inst.operand {
        Operand::Alloca {
            ref tys,
            ref num_elements,
            align,
        } => lower_alloca(ctx, inst.id.unwrap(), tys, num_elements, align),
        Operand::Load {
            ref tys,
            addr,
            align,
        } => lower_load(ctx, inst.id.unwrap(), tys, addr, align),
        Operand::Store {
            ref tys,
            ref args,
            align,
        } => lower_store(ctx, tys, args, align),
        Operand::Ret { val: None, .. } => todo!(),
        Operand::Ret { val: Some(val), ty } => lower_return(ctx, ty, val),
        _ => todo!(),
    }
}

fn lower_alloca(
    ctx: &mut LoweringContext<X86_64>,
    id: InstructionId,
    tys: &[TypeId],
    _num_elements: &ConstantData,
    _align: u32,
) {
    let slot_id = ctx.slots.add_slot(tys[0]);
    ctx.inst_id_to_slot_id.insert(id, slot_id);
}

fn lower_load(
    ctx: &mut LoweringContext<X86_64>,
    id: InstructionId,
    tys: &[TypeId],
    addr: ValueId,
    _align: u32,
) {
    let mut slot = None;

    match ctx.ir_data.value_ref(addr) {
        Value::Instruction(id) => {
            if let Some(slot_id) = ctx.inst_id_to_slot_id.get(id) {
                slot = Some(slot_id);
            }
        }
        _ => todo!(),
    }

    if let Some(slot) = slot {
        if matches!(&*ctx.types.get(tys[0]), Type::Int(32)) {
            let vreg = ctx.vregs.add_vreg_data(tys[0]);
            ctx.inst_id_to_vreg.insert(id, vreg);
            ctx.inst_seq.push(MachInstruction {
                id: None,
                data: InstructionData::MOVrm32 {
                    dst: Either::Right(vreg),
                    src: MemoryOperand::Slot(*slot),
                },
            });
            return;
        }
    }

    todo!()
}

fn lower_store(ctx: &mut LoweringContext<X86_64>, _tys: &[TypeId], args: &[ValueId], _align: u32) {
    let mut slot = None;

    match ctx.ir_data.value_ref(args[1]) {
        Value::Instruction(id) => {
            if let Some(slot_id) = ctx.inst_id_to_slot_id.get(id) {
                slot = Some(slot_id);
            }
        }
        _ => todo!(),
    }

    let mut const_int = None;

    match ctx.ir_data.value_ref(args[0]) {
        Value::Constant(ConstantData::Int(int)) => const_int = Some(*int),
        _ => {}
    }

    match (slot, const_int) {
        (Some(slot), Some(ConstantInt::Int32(imm))) => {
            ctx.inst_seq.append(&mut vec![MachInstruction {
                id: None,
                data: InstructionData::MOVmi32 {
                    dst: MemoryOperand::Slot(*slot),
                    src: imm,
                },
            }]);
            return;
        }
        _ => todo!(),
    }
}

fn lower_return(ctx: &mut LoweringContext<X86_64>, _ty: TypeId, value: ValueId) {
    let value = ctx.ir_data.value_ref(value);
    match value {
        Value::Constant(ConstantData::Int(ConstantInt::Int32(i))) => {
            ctx.inst_seq.push(MachInstruction {
                id: None,
                data: InstructionData::MOVri32 {
                    dst: Either::Left(GR32::EAX.into()),
                    src: *i,
                },
            });
        }
        Value::Instruction(id) => {
            let vreg = ctx.inst_id_to_vreg[id];
            ctx.inst_seq.push(MachInstruction {
                id: None,
                data: InstructionData::MOVrr32 {
                    dst: Either::Left(GR32::EAX.into()),
                    src: Either::Right(vreg),
                },
            });
        }
        _ => todo!(),
    }
    ctx.inst_seq.push(MachInstruction {
        id: None,
        data: InstructionData::RET,
    });
}
