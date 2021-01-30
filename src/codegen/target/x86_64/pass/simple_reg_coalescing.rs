use crate::codegen::{
    function::Function,
    module::Module,
    register::RegisterInfo,
    target::x86_64::{instruction::Opcode, register::RegInfo, X86_64},
};

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
            match inst.data.opcode {
                Opcode::MOVrr32 | Opcode::MOVrr64
                    if RegInfo::to_reg_unit(*inst.data.operands[0].data.as_reg())
                        == RegInfo::to_reg_unit(*inst.data.operands[1].data.as_reg()) =>
                {
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
