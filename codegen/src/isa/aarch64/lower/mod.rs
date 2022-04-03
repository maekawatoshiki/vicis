use super::Aarch64;
use crate::lower::{Lower as LowerTrait, LoweringContext};
use anyhow::Result;
use vicis_core::ir::function::{instruction::Instruction as IrInstruction, Parameter};

#[derive(Clone, Copy)]
pub struct Lower;

impl LowerTrait<Aarch64> for Lower {
    fn lower(_ctx: &mut LoweringContext<Aarch64>, _inst: &IrInstruction) -> Result<()> {
        todo!()
    }

    fn copy_args_to_vregs(
        _ctx: &mut LoweringContext<Aarch64>,
        _params: &[Parameter],
    ) -> Result<()> {
        todo!()
    }
}
