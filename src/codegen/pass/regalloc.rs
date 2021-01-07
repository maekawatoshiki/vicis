use crate::codegen::{
    function::Function, instruction::InstructionData, module::Module, pass::liveness,
    register::VReg, target::Target,
};
use rustc_hash::FxHashSet;
use std::collections::VecDeque;

pub fn run_on_module<T: Target>(module: &mut Module<T>) {
    for (_, func) in &mut module.functions {
        run_on_function(func);
    }
}

// Linear-scan
pub fn run_on_function<T: Target>(function: &mut Function<T>) {
    let mut liveness = liveness::Liveness::new();
    liveness.analyze_function(function);

    // let candidates

    let mut all_vregs = FxHashSet::default();

    for block_id in function.layout.block_iter() {
        for inst_id in function.layout.inst_iter(block_id) {
            let inst = function.data.inst_ref(inst_id);
            for input in inst.data.input_vregs() {
                all_vregs.insert(input);
            }
            for output in inst.data.output_vregs() {
                all_vregs.insert(output);
            }
        }
    }

    debug!(&all_vregs);

    let mut worklist: Vec<VReg> = all_vregs.into_iter().collect();
    worklist.sort_by(|a, b| {
        liveness.vreg_lrs_map[a].0[0]
            .start
            .cmp(&liveness.vreg_lrs_map[b].0[0].start)
    });
    let mut worklist: VecDeque<VReg> = worklist.into_iter().collect();

    while let Some(vreg) = worklist.pop_front() {}

    println!("{:?}", liveness.block_data);
    println!("{:?}", liveness.vreg_lrs_map);
}
