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
    let unaligned_slot_size = function.slots.unaligned_size();
    let num_saved_64bit_regs = 1; // rbp TODO

    let adj = roundup(
        (unaligned_slot_size + num_saved_64bit_regs * 8 + 8/*=call*/) as i32,
        16,
    ) - (num_saved_64bit_regs * 8 + 8) as i32;

    // insert prologue
    if let Some(entry) = function.layout.first_block {
        let sub = function.data.create_inst(Instruction {
            id: None,
            data: InstructionData::SUBr64i32 {
                r: Either::Left(GR64::RSP),
                imm: adj,
            },
        });
        function.layout.insert_inst_at_start(sub, entry);
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
            if !matches!(inst.data, InstructionData::RET) {
                continue;
            }
            epilogues.push((block, inst_id));
        }
    }
    for (block, ret_id) in epilogues {
        let add = function.data.create_inst(Instruction {
            id: None,
            data: InstructionData::ADDr64i32 {
                r: Either::Left(GR64::RSP),
                imm: adj,
            },
        });
        function.layout.insert_inst_before(ret_id, add, block);
        let pop64 = function.data.create_inst(Instruction {
            id: None,
            data: InstructionData::POP64 {
                r: Either::Left(GR64::RBP),
            },
        });
        function.layout.insert_inst_before(ret_id, pop64, block);
    }
}

fn roundup(n: i32, align: i32) -> i32 {
    (n + align - 1) & !(align - 1)
}
