use crate::ir::{function::Function, instruction::Opcode, module::Module};

pub fn run_on_module(module: &mut Module) {
    for (_, function) in module.functions_mut().iter_mut() {
        run_on_function(function);
    }
}

pub fn run_on_function(func: &mut Function) {
    let mut remove_list = vec![];
    for block in func.layout.block_iter() {
        for inst_id in func.layout.inst_iter(block) {
            let inst = func.data.inst_ref(inst_id);
            if matches!(
                inst.opcode,
                Opcode::Call | Opcode::Store | Opcode::Ret | Opcode::Br | Opcode::CondBr
            ) {
                continue;
            }
            if inst.users.len() == 0 {
                remove_list.push(inst_id)
            }
        }
    }
    for inst_id in remove_list {
        func.remove_inst(inst_id).unwrap();
    }
}
