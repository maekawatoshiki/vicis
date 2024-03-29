use crate::{
    function::Function,
    isa::x86_64::{instruction::OperandData, register::GR64, X86_64},
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
    let mut worklist = vec![];

    for block in function.layout.block_iter() {
        for inst_id in function.layout.inst_iter(block) {
            let inst = function.data.inst_ref(inst_id);
            if inst
                .data
                .operands
                .iter()
                .any(|op| matches!(op.data, OperandData::Slot(_)))
            {
                worklist.push(inst_id);
            }
        }
    }

    function.slots.ensure_computed_offsets();

    while let Some(inst_id) = worklist.pop() {
        let mut inst = function.data.instructions[inst_id].clone();

        let mut i = 0;
        let len = inst.data.operands.len();

        while i < len {
            // MemStart indicates the beginning of memory arguments
            if !matches!(&inst.data.operands[i].data, &OperandData::MemStart) {
                i += 1;
                continue;
            }

            i += 1;

            let mem = &mut inst.data.operands[i..i + 6];

            match (&mem[1].data, &mem[2].data) {
                (OperandData::Slot(slot), OperandData::None) => {
                    let off = function.slots.get(*slot).offset;
                    mem[1].data = OperandData::None;
                    mem[2].data = OperandData::Int32(-(off as i32));
                    mem[3].data = OperandData::Reg(GR64::RBP.into());
                }
                (OperandData::Slot(slot), OperandData::Int32(imm)) => {
                    let off = function.slots.get(*slot).offset;
                    mem[2].data = OperandData::Int32(*imm - off as i32);
                    mem[1].data = OperandData::None;
                    mem[3].data = OperandData::Reg(GR64::RBP.into());
                }
                _ => todo!(),
            }

            i += 6;
        }

        function.data.instructions[inst_id] = inst;
    }
}
