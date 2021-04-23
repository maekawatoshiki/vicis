use crate::ir::{
    function::{
        data::Data,
        instruction::{InstructionId, Opcode},
        Function,
    },
    module::Module,
    value::Value,
};

pub fn run_on_module(module: &mut Module) {
    for (_, function) in module.functions_mut().iter_mut() {
        run_on_function(function);
    }
}

pub fn run_on_function(func: &mut Function) {
    let mut worklist = vec![];
    let mut elimination_list = vec![];

    for block in func.layout.block_iter() {
        for inst in func.layout.inst_iter(block) {
            check_if_elimination_possible(&func.data, inst, &mut elimination_list, &mut worklist)
        }
    }

    while let Some(inst) = elimination_list.pop() {
        func.remove_inst(inst).unwrap();
    }

    while let Some(inst) = worklist.pop() {
        check_if_elimination_possible(&func.data, inst, &mut elimination_list, &mut worklist);
        while let Some(inst) = elimination_list.pop() {
            func.remove_inst(inst).unwrap();
        }
    }
}

fn check_if_elimination_possible(
    data: &Data,
    inst: InstructionId,
    elimination_list: &mut Vec<InstructionId>,
    worklist: &mut Vec<InstructionId>,
) {
    let no_users = data.users_of(inst).len() == 0;
    let inst = data.inst_ref(inst);
    let do_not_eliminate = matches!(inst.opcode, Opcode::Alloca | Opcode::Store | Opcode::Call)
        || inst.opcode.is_terminator();
    if do_not_eliminate {
        return;
    }
    if no_users {
        elimination_list.push(inst.id.unwrap());
        for arg in inst
            .operand
            .args()
            .iter()
            .filter_map(|&arg| match data.value_ref(arg) {
                Value::Instruction(id) => Some(*id),
                _ => None,
            })
        {
            worklist.push(arg)
        }
    }
}
