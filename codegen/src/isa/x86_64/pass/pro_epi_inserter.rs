use crate::{
    function::{instruction::Instruction, Function},
    isa::x86_64::{
        instruction::{InstructionData, Opcode, Operand},
        register::{RegClass, GR64},
        X86_64,
    },
    module::Module,
};
use anyhow::Result;

pub fn run_on_module(module: &mut Module<X86_64>) -> Result<()> {
    for (_, func) in &mut module.functions {
        run_on_function(func);
    }
    Ok(())
}

pub fn run_on_function(function: &mut Function<X86_64>) {
    if function.is_declaration {
        return;
    }

    let slot_size = function.slots.ensure_computed_offsets();

    let mut used_csr = function
        .data
        .used_csr
        .clone()
        .into_iter()
        .map(|r| r.apply(&RegClass::GR64))
        .collect::<Vec<_>>();
    used_csr.sort();
    let num_saved_64bit_regs = 1/*8=rbp*/ + used_csr.len() as u32;

    let adj = roundup(
        (slot_size + num_saved_64bit_regs * 8 + 8/*=call*/) as i32,
        16,
    ) - (num_saved_64bit_regs * 8 + 8) as i32;

    // insert prologue
    let entry = function.layout.first_block.unwrap();
    if adj > 0 {
        let sub = function.data.create_inst(Instruction::new(
            InstructionData {
                opcode: Opcode::SUBr64i32,
                operands: vec![
                    Operand::input_output(GR64::RSP.into()),
                    Operand::input(adj.into()),
                ],
            },
            entry,
        ));
        function.layout.insert_inst_at_start(sub, entry);
    }
    let mov = function.data.create_inst(Instruction::new(
        InstructionData {
            opcode: Opcode::MOVrr64,
            operands: vec![
                Operand::output(GR64::RBP.into()),
                Operand::input(GR64::RSP.into()),
            ],
        },
        entry,
    ));
    function.layout.insert_inst_at_start(mov, entry);
    for r in &used_csr {
        let push64 = function.data.create_inst(Instruction::new(
            InstructionData {
                opcode: Opcode::PUSH64,
                operands: vec![Operand::input(r.into())],
            },
            entry,
        ));
        function.layout.insert_inst_at_start(push64, entry);
    }
    let push64 = function.data.create_inst(Instruction::new(
        InstructionData {
            opcode: Opcode::PUSH64,
            operands: vec![Operand::input(GR64::RBP.into())],
        },
        entry,
    ));
    function.layout.insert_inst_at_start(push64, entry);

    // insert epilogue
    let mut epilogues = vec![];
    for block in function.layout.block_iter() {
        for inst_id in function.layout.inst_iter(block) {
            let inst = function.data.inst_ref(inst_id);
            if !matches!(inst.data.opcode, Opcode::RET) {
                continue;
            }
            epilogues.push((block, inst_id));
        }
    }
    for (block, ret_id) in epilogues {
        if adj > 0 {
            let add = function.data.create_inst(Instruction::new(
                InstructionData {
                    opcode: Opcode::ADDr64i32,
                    operands: vec![
                        Operand::output(GR64::RSP.into()),
                        Operand::input(adj.into()),
                    ],
                },
                block,
            ));
            function.layout.insert_inst_before(ret_id, add, block);
        }
        for r in &used_csr {
            let pop64 = function.data.create_inst(Instruction::new(
                InstructionData {
                    opcode: Opcode::POP64,
                    operands: vec![Operand::input(r.into())],
                },
                entry,
            ));
            function.layout.insert_inst_before(ret_id, pop64, block);
        }
        let pop64 = function.data.create_inst(Instruction::new(
            InstructionData {
                opcode: Opcode::POP64,
                operands: vec![Operand::input(GR64::RBP.into())],
            },
            block,
        ));
        function.layout.insert_inst_before(ret_id, pop64, block);
    }
}

fn roundup(n: i32, align: i32) -> i32 {
    (n + align - 1) & !(align - 1)
}
