use crate::{
    function::{
        instruction::{InstructionId, TargetInst},
        Function,
    },
    isa::TargetIsa,
    module::Module,
    pass::liveness,
    pass::spiller,
    register::{Reg, RegisterClass, RegisterInfo, VReg},
};
use anyhow::Result;
use rustc_hash::{FxHashMap, FxHashSet};
use std::collections::VecDeque;

pub fn run_on_module<T: TargetIsa>(module: &mut Module<T>) -> Result<()> {
    for (_, func) in &mut module.functions {
        run_on_function(func);
    }
    Ok(())
}

// Linear-scan
pub fn run_on_function<T: TargetIsa>(function: &mut Function<T>) {
    let mut liveness = liveness::Liveness::<T>::new();
    liveness.analyze_function(function);

    let mut all_vregs = FxHashSet::default();
    for block_id in function.layout.block_iter() {
        for inst_id in function.layout.inst_iter(block_id) {
            let inst = function.data.inst_ref(inst_id);
            for r in inst.data.all_vregs() {
                all_vregs.insert(r);
            }
        }
    }

    // Insert spill and reload code around call site
    for vreg in collect_vregs_alive_around_call(function, &liveness, &all_vregs) {
        let mut new_vregs = vec![];
        spiller::Spiller::new(function, &mut liveness).spill(vreg, &mut new_vregs);
        all_vregs.remove(&vreg);
        all_vregs.extend(new_vregs.into_iter())
    }

    let preferred = collect_preferred_registers(function, &all_vregs);

    debug!(&all_vregs);
    debug!(&function);

    let mut worklist: Vec<VReg> = all_vregs.into_iter().collect();
    // sort by segment start
    worklist.sort_by(|a, b| {
        liveness
            .vreg_range(a)
            .unwrap()
            .first_seg()
            .unwrap()
            .start
            .cmp(&liveness.vreg_range(b).unwrap().first_seg().unwrap().start)
    });
    let mut worklist: VecDeque<VReg> = worklist.into_iter().collect();

    let mut assigned_regs: FxHashMap<VReg, Reg> = FxHashMap::default();

    // TODO: Refactoring.
    // TODO: Implement a brand new regalloc.
    let mut new_vregs = vec![];
    while let Some(vreg) = worklist.pop_front() {
        let mut availables =
            T::RegClass::for_type(&function.types, function.data.vregs.type_for(vreg)).gpr_list();
        let _ = availables.pop(); // reserved for spill.

        if let Some(preferred) = preferred.get(&vreg) {
            availables.splice(0..0, preferred.clone());
        }

        let mut allocated = false;
        for reg in availables {
            let reg_unit = T::RegInfo::to_reg_unit(reg);
            if !liveness.interfere(reg_unit, vreg) {
                assigned_regs.insert(vreg, reg);
                liveness.assign(reg_unit, vreg);
                allocated = true;
                break;
            }
        }
        if !allocated {
            log::debug!("spill: {:?}", vreg);
            spiller::Spiller::new(function, &mut liveness).spill(vreg, &mut new_vregs);
        }
    }

    // Allocate spill register.
    let mut worklist: VecDeque<VReg> = new_vregs.into_iter().collect();
    while let Some(vreg) = worklist.pop_front() {
        let reg = T::RegClass::for_type(&function.types, function.data.vregs.type_for(vreg))
            .gpr_list()
            .pop()
            .unwrap();
        let reg_unit = T::RegInfo::to_reg_unit(reg);
        assert!(!liveness.interfere(reg_unit, vreg));
        assigned_regs.insert(vreg, reg);
        liveness.assign(reg_unit, vreg);
    }

    // Rewrite vreg for reg
    for block_id in function.layout.block_iter() {
        for inst_id in function.layout.inst_iter(block_id) {
            let inst = function.data.inst_ref_mut(inst_id);
            // println!("{:?}", inst.data);
            for vreg in inst.data.all_vregs() {
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

pub fn collect_vregs_alive_around_call<T: TargetIsa>(
    function: &mut Function<T>,
    liveness: &liveness::Liveness<T>,
    all_vregs: &FxHashSet<VReg>,
) -> Vec<VReg> {
    let mut output = vec![];
    for block_id in function.layout.block_iter() {
        for inst_id in function.layout.inst_iter(block_id) {
            let inst = function.data.inst_ref(inst_id);
            if !inst.data.is_call() {
                continue;
            }
            let call_lr = liveness::LiveSegment::new_point(liveness.inst_to_pp[&inst_id]);
            for vreg in all_vregs {
                let vreg_lrs = &liveness.vreg_lrs_map[vreg];
                if vreg_lrs.interfere_with_segment(&call_lr) {
                    output.push(*vreg);
                }
            }
        }
    }
    output
}

pub fn collect_preferred_registers<T: TargetIsa>(
    function: &mut Function<T>,
    all_vregs: &FxHashSet<VReg>,
) -> FxHashMap<VReg, Vec<Reg>> {
    let mut preferred = FxHashMap::default();
    for &vreg in all_vregs {
        let mut visited = FxHashSet::default();
        let rs = find_preferred_registers(function, vreg, &mut visited);
        if !rs.is_empty() {
            preferred.insert(vreg, rs);
        }
    }
    preferred
}

pub fn find_preferred_registers<T: TargetIsa>(
    function: &Function<T>,
    vreg: VReg,
    visited: &mut FxHashSet<InstructionId<T::Inst>>,
) -> Vec<Reg> {
    let mut list = vec![];
    let users = function.data.vreg_users.get(vreg);
    for user in users {
        let inst = function.data.inst_ref(user.inst_id);
        if !visited.insert(user.inst_id) {
            continue;
        }
        if user.write {
            continue;
        }
        if inst.data.is_copy() {
            let regs = inst.data.output_regs();
            if !regs.is_empty() {
                list.insert(0, regs[0]);
                continue;
            }

            let regs = inst.data.output_vregs();
            if !regs.is_empty() {
                let dst = regs[0]; // TODO: Why do we know the first element of `regs` is destination?
                list.extend(find_preferred_registers(function, dst, visited).into_iter());
            }
        }
    }
    list
}
