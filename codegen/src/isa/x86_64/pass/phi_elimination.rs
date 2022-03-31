use crate::{
    function::{basic_block::BasicBlockId, instruction::Instruction, Function},
    isa::x86_64::{
        instruction::{InstructionData, Opcode, Operand, OperandData},
        X86_64,
    },
    module::Module,
    register::Reg,
};
use anyhow::Result;
use rustc_hash::FxHashMap;

pub fn run_on_module(module: &mut Module<X86_64>) -> Result<()> {
    for (_, func) in &mut module.functions {
        run_on_function(func);
    }
    Ok(())
}

pub fn run_on_function(function: &mut Function<X86_64>) {
    let mut worklist = vec![];
    let mut map: FxHashMap<Reg, Vec<(OperandData, BasicBlockId)>> = FxHashMap::default();

    for block_id in function.layout.block_iter() {
        for inst_id in function.layout.inst_iter(block_id) {
            let inst = function.data.inst_ref(inst_id);
            if !matches!(inst.data.opcode, Opcode::Phi) {
                continue;
            }
            worklist.push(inst_id);
            let output = *inst.data.operands[0].data.as_reg();
            for i in (0..inst.data.operands[1..].len()).step_by(2) {
                let val = inst.data.operands[1 + i /*+ 0*/].data.clone();
                let block = *inst.data.operands[1 + i + 1].data.as_block();
                map.entry(output)
                    .or_insert_with(Vec::new)
                    .push((val, block));
            }
        }
    }

    for (output, args) in map {
        for (arg, block) in args {
            let maybe_term = function.layout.last_inst_of(block).unwrap();
            // assert!(matches!(arg, OperandData::Int32(_)));
            let copy = match arg {
                OperandData::Int32(_) => Instruction::new(
                    InstructionData {
                        opcode: Opcode::MOVri32,
                        operands: vec![
                            Operand::output(OperandData::Reg(output)),
                            Operand::new(arg),
                        ],
                    },
                    block,
                ),
                OperandData::Reg(_) => Instruction::new(
                    InstructionData {
                        opcode: Opcode::MOVrr32,
                        operands: vec![
                            Operand::output(OperandData::Reg(output)),
                            Operand::input(arg),
                        ],
                    },
                    block,
                ),
                _ => todo!(),
            };
            let copy = function.data.create_inst(copy);
            function.layout.insert_inst_before(maybe_term, copy, block);
        }
    }

    for inst_id in worklist {
        function.remove_inst(inst_id);
    }
}
