use super::liveness::{Liveness, ProgramPoint};
use crate::{
    function::{
        basic_block::BasicBlockId,
        instruction::{InstructionId, TargetInst},
        slot::SlotId,
        Function,
    },
    isa::TargetIsa,
    register::VReg,
};

pub struct Spiller<'a, 'b, T: TargetIsa> {
    function: &'a mut Function<'b, T>,
    liveness: &'a mut Liveness<T>,
}

impl<'a, 'b, T: TargetIsa> Spiller<'a, 'b, T> {
    pub fn new(function: &'a mut Function<'b, T>, liveness: &'a mut Liveness<T>) -> Self {
        Self { function, liveness }
    }

    pub fn spill(&mut self, vreg: VReg, new_vregs: &mut Vec<VReg>) {
        let ty = self.function.data.vregs.type_for(vreg);
        let dl = self.function.isa.data_layout();
        let sz = dl.get_size_of(&self.function.types, ty) as u32;
        let align = dl.get_align_of(&self.function.types, ty) as u32;
        let slot = self.function.slots.add_slot(ty, sz, align);

        self.insert_spill(vreg, slot, new_vregs);
        self.insert_reload(vreg, slot, new_vregs);

        // create live ranges for new virtual registers
        for &mut new_vreg in new_vregs {
            self.liveness.compute_live_ranges(self.function, new_vreg)
        }

        self.liveness.remove_vreg(vreg);
    }

    fn insert_spill(&mut self, vreg: VReg, slot: SlotId, new_vregs: &mut Vec<VReg>) {
        let mut defs = vec![];
        for user in self.function.data.vreg_users.get(vreg) {
            if user.write {
                defs.push(user.inst_id)
            }
        }

        if defs.is_empty() {
            return;
        }

        let new_vreg = self.function.data.vregs.create_from(vreg);
        new_vregs.push(new_vreg);

        // Most cases
        if defs.len() == 1 {
            let def_id = *defs.get(0).unwrap();
            let def_block;
            {
                let inst = &mut self.function.data.instructions[def_id];
                def_block = inst.parent;
                inst.replace_vreg(&mut self.function.data.vreg_users, vreg, new_vreg);
            }
            let inst = T::Inst::store_vreg_to_slot(self.function, new_vreg, slot, def_block);
            let inst = self.function.data.create_inst(inst);
            self.insert_inst_after(def_id, inst, def_block);
            return;
        }

        // Two addr instruction
        if defs.len() == 2 {
            let mut def_id = None;
            let mut def_block = None;
            for &id in &defs {
                let inst = &mut self.function.data.instructions[id];
                if !inst.data.is_copy() {
                    def_id = Some(id);
                    def_block = Some(inst.parent);
                }
                inst.replace_vreg(&mut self.function.data.vreg_users, vreg, new_vreg);
            }
            let def_id = def_id.unwrap();
            let def_block = def_block.unwrap();
            let inst = T::Inst::store_vreg_to_slot(self.function, new_vreg, slot, def_block);
            let inst = self.function.data.create_inst(inst);
            self.insert_inst_after(def_id, inst, def_block);
            return;
        }

        panic!("invalid defs len: {}", defs.len())
    }

    fn insert_reload(&mut self, vreg: VReg, slot: SlotId, new_vregs: &mut Vec<VReg>) {
        let mut uses = vec![];
        for user in self.function.data.vreg_users.get(vreg) {
            if user.read {
                uses.push(user.inst_id)
            }
        }

        if uses.is_empty() {
            return;
        }

        let new_vreg = self.function.data.vregs.create_from(vreg);
        new_vregs.push(new_vreg);

        for use_id in uses {
            let use_block;
            {
                let inst = &mut self.function.data.instructions[use_id];
                use_block = inst.parent;
                inst.replace_vreg(&mut self.function.data.vreg_users, vreg, new_vreg);
            }
            let inst = T::Inst::load_from_slot(self.function, new_vreg, slot, use_block);
            let inst = self.function.data.create_inst(inst);
            self.insert_inst_before(use_id, inst, use_block);
        }
    }

    fn insert_inst_after(
        &mut self,
        after: InstructionId<T::Inst>,
        inst: InstructionId<T::Inst>,
        block: BasicBlockId,
    ) {
        let after_pp = self.liveness.inst_to_pp[&after];
        let next_after = self.function.layout.next_inst_of(after).unwrap();
        let next_after_pp = self.liveness.inst_to_pp[&next_after];
        if let Some(inst_pp) = ProgramPoint::between(after_pp, next_after_pp) {
            self.liveness.inst_to_pp.insert(inst, inst_pp);
            self.function.layout.insert_inst_after(after, inst, block);
        } else {
            self.liveness.recompute_program_points_after(after_pp);
            self.insert_inst_after(after, inst, block);
        }
    }

    fn insert_inst_before(
        &mut self,
        before: InstructionId<T::Inst>,
        inst: InstructionId<T::Inst>,
        block: BasicBlockId,
    ) {
        let before_pp = self.liveness.inst_to_pp[&before];
        let prev_before = self.function.layout.prev_inst_of(before).unwrap();
        let prev_before_pp = self.liveness.inst_to_pp[&prev_before];
        if let Some(inst_pp) = ProgramPoint::between(prev_before_pp, before_pp) {
            self.liveness.inst_to_pp.insert(inst, inst_pp);
            self.function.layout.insert_inst_before(before, inst, block);
        } else {
            self.liveness.recompute_program_points_after(before_pp);
            self.insert_inst_before(before, inst, block)
        }
    }
}
