use crate::codegen::{
    function::Function,
    module::Module,
    target::{
        x86_64::{instruction::InstructionData, X86_64},
        Target,
    },
};
use either::Either;

pub fn run_on_module(module: &mut Module<X86_64>) {
    for (_, func) in &mut module.functions {
        run_on_function(func);
    }
}

pub fn run_on_function(function: &mut Function<X86_64>) {
    let mut worklist = vec![];

    for block_id in function.layout.block_iter() {
        for inst_id in function.layout.inst_iter(block_id) {
            let inst = function.data.inst_ref(inst_id);
            match inst.data {
                InstructionData::MOVrr32 {
                    dst: Either::Left(dst),
                    src: Either::Left(src),
                } if function.target.to_reg_unit(dst) == function.target.to_reg_unit(src) => {
                    worklist.push(inst_id)
                }
                InstructionData::MOVrr64 {
                    dst: Either::Left(dst),
                    src: Either::Left(src),
                } if function.target.to_reg_unit(dst) == function.target.to_reg_unit(src) => {
                    worklist.push(inst_id)
                }
                _ => {}
            }
        }
    }

    for inst_id in worklist {
        function.remove_inst(inst_id);
    }
}
