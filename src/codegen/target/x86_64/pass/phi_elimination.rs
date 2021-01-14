use crate::codegen::{
    calling_conv::CallingConv,
    function::{basic_block::BasicBlockId, instruction::Instruction, Function},
    module::Module,
    register::VReg,
    target::x86_64::{
        instruction::{InstructionData, Opcode, Operand, OperandData},
        register::RegClass,
        X86_64,
    },
};
use rustc_hash::FxHashMap;

pub fn run_on_module<CC: CallingConv<RegClass>>(module: &mut Module<X86_64<CC>>) {
    for (_, func) in &mut module.functions {
        run_on_function(func);
    }
}

pub fn run_on_function<CC: CallingConv<RegClass>>(function: &mut Function<X86_64<CC>>) {
    let mut worklist = vec![];
    let mut map: FxHashMap<VReg, Vec<(OperandData, BasicBlockId)>> = FxHashMap::default();

    for block_id in function.layout.block_iter() {
        for inst_id in function.layout.inst_iter(block_id) {
            let inst = function.data.inst_ref(inst_id);
            if !matches!(inst.data.opcode, Opcode::Phi) {
                continue;
            }
            worklist.push(inst_id);
            let output = *inst.data.operands[0].data.as_vreg();
            for i in (0..inst.data.operands[1..].len()).step_by(2) {
                let val = inst.data.operands[1 + i + 0].data.clone();
                let block = *inst.data.operands[1 + i + 1].data.as_block();
                map.entry(output).or_insert(vec![]).push((val, block));
            }
        }
    }

    for (output, args) in map {
        for (arg, block) in args {
            let maybe_term = function.layout.last_inst_of(block).unwrap();
            // assert!(matches!(arg, OperandData::Int32(_)));
            let copy = match arg {
                OperandData::Int32(_) => Instruction::new(InstructionData {
                    opcode: Opcode::MOVri32,
                    operands: vec![
                        Operand::output(OperandData::VReg(output)),
                        Operand::new(arg),
                    ],
                }),
                OperandData::VReg(_) => Instruction::new(InstructionData {
                    opcode: Opcode::MOVrr32,
                    operands: vec![
                        Operand::output(OperandData::VReg(output)),
                        Operand::input(arg),
                    ],
                }),
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