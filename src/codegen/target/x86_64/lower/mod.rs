use crate::codegen::{
    instruction::Instruction as MachInstruction,
    lower::pattern::{Lower as LowerTrait, LoweringContext},
    target::x86_64::{instruction::InstructionData, register::GR32, X86_64},
};
use crate::ir::{
    instruction::{Instruction as IrInstruction, InstructionId, Operand},
    types::TypeId,
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
    fn lower(
        &self,
        ctx: &mut LoweringContext<X86_64>,
        inst: &IrInstruction,
    ) -> Vec<MachInstruction<InstructionData>> {
        match inst.operand {
            Operand::Alloca {
                ref tys,
                ref num_elements,
                align,
            } => lower_alloca(ctx, inst.id.unwrap(), tys, num_elements, align),
            Operand::Ret { val: None, .. } => todo!(),
            Operand::Ret { val: Some(val), ty } => lower_return(ctx, ty, val),
            _ => todo!(),
        }
    }
}

fn lower_alloca(
    ctx: &mut LoweringContext<X86_64>,
    id: InstructionId,
    tys: &[TypeId],
    _num_elements: &ConstantData,
    _align: u32,
) -> Vec<MachInstruction<InstructionData>> {
    let slot_id = ctx.slots.add_slot(tys[0]);
    ctx.inst_id_to_slot_id.insert(id, slot_id);
    vec![]
}

fn lower_return(
    ctx: &mut LoweringContext<X86_64>,
    _ty: TypeId,
    value: ValueId,
) -> Vec<MachInstruction<InstructionData>> {
    let value = ctx.ir_data.value_ref(value);
    match value {
        Value::Constant(ConstantData::Int(ConstantInt::Int32(i))) => {
            return vec![
                MachInstruction {
                    id: None,
                    data: InstructionData::MOVri32 {
                        dst: Either::Left(GR32::EAX),
                        src: *i,
                    },
                },
                MachInstruction {
                    id: None,
                    data: InstructionData::RET,
                },
            ];
        }
        _ => todo!(),
    }
}
