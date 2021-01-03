use crate::codegen::{
    function::Function,
    instruction::Instruction,
    module::Module,
    target::x86_64::{instruction::InstructionData, register::GR64, X86_64},
};
use either::Either;

pub fn run_on_module(module: &mut Module<X86_64>) {
    for (_, func) in &mut module.functions {
        run_on_function(func);
    }
}

pub fn run_on_function(function: &mut Function<X86_64>) {
    // insert prologue
    if let Some(entry) = function.layout.first_block {
        let push64 = function.data.create_inst(Instruction {
            id: None,
            data: InstructionData::PUSH64 {
                r: Either::Left(GR64::RBP),
            },
        });
        function.layout.insert_inst_at_start(push64, entry);
    }

    // insert epilogue
    let mut epilogues = vec![];
    for block in function.layout.block_iter() {
        for inst_id in function.layout.inst_iter(block) {
            let inst = function.data.inst_ref(inst_id);
            // println!("{:?}", inst);
            if !matches!(inst.data, InstructionData::RET) {
                continue;
            }
            epilogues.push((block, inst_id));
        }
    }
    for (block, ret_id) in epilogues {
        let pop64 = function.data.create_inst(Instruction {
            id: None,
            data: InstructionData::POP64 {
                r: Either::Left(GR64::RBP),
            },
        });
        function.layout.insert_inst_before(ret_id, pop64, block);
    }
}
