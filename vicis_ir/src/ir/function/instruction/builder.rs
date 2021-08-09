use super::{InstructionId, Opcode, Operand, Ret};
use crate::ir::{function::builder::Builder as FuncBuilder, value::ValueId};
// use rustc_hash::FxHashSet;

pub struct Builder<'a: 'short, 'short> {
    func_builder: &'short mut FuncBuilder<'a>,
}

impl<'a: 'short, 'short> Builder<'a, 'short> {
    pub fn new(func_builder: &'short mut FuncBuilder<'a>) -> Self {
        Self { func_builder }
    }

    pub fn ret(&mut self, val: ValueId) -> InstructionId {
        let cur_block = self.func_builder.cur_block.unwrap();
        let ty = self.func_builder.func.result_ty;
        let inst = Opcode::Ret
            .with_block(cur_block)
            .with_operand(Operand::Ret(Ret { ty, val: Some(val) }));
        let inst = self.func_builder.func.data.create_inst(inst);
        self.func_builder.func.layout.append_inst(inst, cur_block);
        inst
    }
}
