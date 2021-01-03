use crate::codegen::{
    inst_selection::{pattern::ir, Context},
    instruction::Instruction as MachInstruction,
    target::x86_64::{instruction::InstructionData, register::GR32},
};
use crate::ir::{function::Data as IrData, instruction::Instruction as IrInstruction};
use either::Either;

pub fn ret<'a>(ctx: Context<InstructionData>) -> Option<Vec<MachInstruction<InstructionData>>> {
    if let Some(imm32) = ir::ret(ir::any_i32())(ctx.ir_data, ctx.inst) {
        return Some(vec![
            MachInstruction {
                id: None,
                data: InstructionData::MOVri32 {
                    dst: Either::Left(GR32::EAX),
                    src: *imm32,
                },
            },
            MachInstruction {
                id: None,
                data: InstructionData::RET,
            },
        ]);
    }
    None
}
