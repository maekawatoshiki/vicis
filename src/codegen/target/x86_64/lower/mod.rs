use crate::codegen::{
    instruction::Instruction as MachInstruction,
    lower::pattern::{Lower as LowerTrait, LoweringContext},
    target::x86_64::{instruction::InstructionData, register::GR32},
};
use crate::ir::{
    instruction::{Instruction as IrInstruction, Operand},
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

impl LowerTrait<InstructionData> for Lower {
    fn lower(
        &self,
        ctx: &mut LoweringContext<InstructionData>,
        inst: &IrInstruction,
    ) -> Vec<MachInstruction<InstructionData>> {
        match inst.operand {
            Operand::Ret { val: None, .. } => todo!(),
            Operand::Ret { val: Some(val), ty } => lower_return(ctx, ty, val),
            _ => todo!(),
        }
    }
}

fn lower_return(
    ctx: &mut LoweringContext<InstructionData>,
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
