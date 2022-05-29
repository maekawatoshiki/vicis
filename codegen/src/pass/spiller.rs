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

        // Create live ranges for new virtual registers.
        for &mut new_vreg in new_vregs {
            self.liveness.compute_live_ranges(self.function, new_vreg)
        }

        self.liveness.remove_vreg(vreg);
    }

    fn insert_spill(&mut self, vreg: VReg, slot: SlotId, new_vregs: &mut Vec<VReg>) {
        let mut defs = self
            .function
            .data
            .vreg_users
            .get(vreg)
            .into_iter()
            .filter_map(|user| if user.write { Some(user.inst_id) } else { None })
            .collect::<Vec<_>>();
        // e.g. If 'MOV' and 'ADD' below are contained in `defs`,
        //      remove 'MOV' from `defs` to avoid an unnecessary spill.
        // 1: MOV a, b
        // 2: ADD a, c
        defs.sort_by(|a, b| {
            let inst2pp = &self.liveness.inst_to_pp;
            inst2pp[b].cmp(&inst2pp[a])
        });
        defs.dedup_by(|a, b| {
            self.function
                .layout
                .prev_inst_of(*a)
                .map_or(false, |a| a == *b)
        });

        if defs.is_empty() {
            return;
        }

        for &def_id in &defs {
            let new_vreg = self.function.data.vregs.create_from(vreg);
            new_vregs.push(new_vreg);
            let def_block;
            {
                let def_inst = &mut self.function.data.instructions[def_id];
                def_inst.replace_vreg(&mut self.function.data.vreg_users, vreg, new_vreg);
                def_block = def_inst.parent;
            }
            let inst = T::Inst::store_vreg_to_slot(self.function, new_vreg, slot, def_block);
            let inst = self.function.data.create_inst(inst);
            self.insert_inst_after(def_id, inst, def_block);
        }
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

        for use_id in uses {
            let new_vreg = self.function.data.vregs.create_from(vreg);
            new_vregs.push(new_vreg);
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
            self.liveness
                .recompute_program_points_after(after_pp, false);
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
        let prev_before_pp = if let Some(prev_before) = self.function.layout.prev_inst_of(before) {
            self.liveness.inst_to_pp[&prev_before]
        } else {
            self.liveness
                .recompute_program_points_after(before_pp, true);
            ProgramPoint(before_pp.0, 0)
        };
        if let Some(inst_pp) = ProgramPoint::between(prev_before_pp, before_pp) {
            self.liveness.inst_to_pp.insert(inst, inst_pp);
            self.function.layout.insert_inst_before(before, inst, block);
        } else {
            self.liveness
                .recompute_program_points_after(before_pp, false);
            self.insert_inst_before(before, inst, block)
        }
    }
}
