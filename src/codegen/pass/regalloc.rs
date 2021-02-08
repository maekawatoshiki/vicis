use crate::codegen::{
    function::{instruction::InstructionData, Function},
    isa::Target,
    module::Module,
    pass::liveness,
    register::{Reg, RegisterClass, RegisterInfo, VReg},
};
use rustc_hash::{FxHashMap, FxHashSet};
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

    let mut assigned_regs: FxHashMap<VReg, Reg> = FxHashMap::default();

    while let Some(vreg) = worklist.pop_front() {
        let availables =
            T::RegClass::for_type(&function.types, function.vregs.type_for(vreg)).gpr_list();
        for reg in availables {
            let reg_unit = T::RegInfo::to_reg_unit(reg);
            let lrs1 = &liveness.vreg_lrs_map[&vreg];
            let lrs2 = liveness
                .reg_lrs_map
                .entry(reg_unit)
                .or_insert(liveness::LiveRanges(vec![]));
            // println!("{:?}", vreg);
            if !lrs1.interfere(lrs2) {
                // assign reg for vreg
                assigned_regs.insert(vreg, reg);
                lrs2.merge(lrs1);
                break;
            }
        }
    }

    // Rewrite vreg for reg
    for block_id in function.layout.block_iter() {
        for inst_id in function.layout.inst_iter(block_id) {
            let inst = function.data.inst_ref_mut(inst_id);
            // println!("{:?}", inst.data);
            for vreg in inst
                .data
                .input_vregs()
                .into_iter()
                .chain(inst.data.output_vregs().into_iter())
            {
                if let Some(reg) = assigned_regs.get(&vreg) {
                    // println!("{:?} => {:?}", vreg, reg);
                    inst.data.rewrite(vreg, *reg);
                }
            }
        }
    }

    debug!(liveness.block_data);
    debug!(liveness.vreg_lrs_map);
    debug!(liveness.reg_lrs_map);
}
