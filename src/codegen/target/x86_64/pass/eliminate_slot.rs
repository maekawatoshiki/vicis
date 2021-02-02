use crate::codegen::{
    function::Function,
    module::Module,
    target::x86_64::{instruction::MemoryOperand, register::GR64, X86_64},
};
use rustc_hash::FxHashMap;

pub fn run_on_module(module: &mut Module<X86_64>) {
    for (_, func) in &mut module.functions {
        run_on_function(func);
    }
}

pub fn run_on_function(function: &mut Function<X86_64>) {
    let mut worklist = vec![];

    for block in function.layout.block_iter() {
        for inst_id in function.layout.inst_iter(block) {
            let inst = function.data.inst_ref(inst_id);
            if inst.data.mem_ops().len() > 0 {
                worklist.push(inst_id);
            }
        }
    }

    let mut offset = 0;
    let mut offset_map = FxHashMap::default();

    while let Some(inst_id) = worklist.pop() {
        let inst = function.data.inst_ref_mut(inst_id);
        for mem_op in inst.data.mem_ops_mut() {
            match mem_op {
                // TODO: Refactoring
                MemoryOperand::Slot(id) => {
                    if let Some(offset) = offset_map.get(id) {
                        *mem_op = MemoryOperand::ImmReg(-(*offset as i32), GR64::RBP.into());
                    } else {
                        offset += function.slots.get(*id).size;
                        offset_map.insert(*id, offset);
                        *mem_op = MemoryOperand::ImmReg(-(offset as i32), GR64::RBP.into());
                    }
                }
                MemoryOperand::ImmSlot(imm, id) => {
                    if let Some(offset) = offset_map.get(id) {
                        *mem_op = MemoryOperand::ImmReg(*imm - (*offset as i32), GR64::RBP.into());
                    } else {
                        offset += function.slots.get(*id).size;
                        offset_map.insert(*id, offset);
                        *mem_op = MemoryOperand::ImmReg(*imm - (offset as i32), GR64::RBP.into());
                    }
                }
                _ => {}
            }
        }
    }
}
