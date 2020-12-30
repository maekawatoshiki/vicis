use crate::ir::{
    function::FunctionId,
    instruction::{Instruction, Operand},
    module::Module,
};

pub struct Interpreter<'a> {
    module: &'a Module,
}

impl<'a> Interpreter<'a> {
    pub fn new(module: &'a Module) -> Self {
        Self { module }
    }

    pub fn run_function(&mut self, func_id: FunctionId) {
        let func = &self.module.functions()[func_id];
        for block in func.layout.block_iter() {
            for inst in func
                .layout
                .inst_iter(block)
                .into_iter()
                .map(|id| func.data.inst_ref(id))
            {
                match &inst.operand {
                    Operand::Alloca {
                        tys,
                        num_elements,
                        align,
                    } => {
                        let alloc_ty = tys[0];
                    }
                    _ => {}
                }
            }
        }
    }
}
