use crate::codegen::{
    function::{
        basic_block::BasicBlockId,
        instruction::{Instruction, InstructionData as ID, InstructionId, InstructionInfo as II},
        Function,
    },
    isa::TargetIsa,
    // module::Module,
    register::{RegUnit, RegisterInfo, VReg},
};
use rustc_hash::{FxHashMap, FxHashSet};
use std::cmp::Ordering;

pub struct Liveness<T: TargetIsa> {
    pub block_data: FxHashMap<BasicBlockId, BlockData>,
    pub vreg_lrs_map: FxHashMap<VReg, LiveRanges>,
    pub reg_lrs_map: FxHashMap<RegUnit, LiveRanges>,
    pub inst_to_pp: FxHashMap<InstructionId<<T::InstInfo as II>::Data>, ProgramPoint>,
}

#[derive(Debug, Clone)]
pub struct LiveRanges(pub Vec<LiveRange>);

#[derive(Debug, Clone)]
pub struct LiveRange {
    pub start: ProgramPoint,
    pub end: ProgramPoint,
}

#[derive(Debug)]
pub struct BlockData {
    vreg_def: FxHashSet<VReg>,
    vreg_live_in: FxHashSet<VReg>,
    vreg_live_out: FxHashSet<VReg>,
    reg_def: FxHashSet<RegUnit>,
    reg_live_in: FxHashSet<RegUnit>,
    reg_live_out: FxHashSet<RegUnit>,
}

// pub type ProgramPointId = Id<ProgramPoint>;

#[derive(Debug, Clone, Copy)]
pub struct ProgramPoint(pub u32, pub u32);

const STEP: u32 = 16;

impl PartialOrd for ProgramPoint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.0 < other.0 {
            return Some(Ordering::Less);
        }

        if self.0 > other.0 {
            return Some(Ordering::Greater);
        }

        assert_eq!(self.0, other.0);

        if self.1 < other.1 {
            return Some(Ordering::Less);
        }

        if self.1 == other.1 {
            return Some(Ordering::Equal);
        }

        assert!(self.1 > other.1);

        Some(Ordering::Greater)
    }
}

impl Ord for ProgramPoint {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialEq for ProgramPoint {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

impl Eq for ProgramPoint {}

impl ProgramPoint {
    pub fn between(prev: Self, next: Self) -> Option<Self> {
        if prev.0 != next.0 {
            return None;
        }
        if prev.1 > next.1 {
            return None;
        }
        if next.1 - prev.1 <= 1 {
            return None;
        }
        let new = (next.1 + prev.1) / 2;
        Some(Self(prev.0, new))
    }
}

// pub fn run_on_module<T: TargetIsa>(module: &mut Module<T>) {
//     for (_, func) in &mut module.functions {
//         run_on_function(func);
//     }
// }

// pub fn run_on_function<T: TargetIsa>(function: &mut Function<T>) -> Liveness {
//     // for block_id in function.layout.block_iter() {
//     //     for inst_id in function.layout.inst_iter(block_id) {
//     //         let inst = function.data.inst_ref(inst_id);
//     //     }
//     // }
//     todo!()
// }

impl<T: TargetIsa> Liveness<T> {
    pub fn new() -> Self {
        Self {
            block_data: FxHashMap::default(),
            vreg_lrs_map: FxHashMap::default(),
            reg_lrs_map: FxHashMap::default(),
            inst_to_pp: FxHashMap::default(),
        }
    }

    pub fn analyze_function(&mut self, func: &Function<T>) {
        // Analyze live-in and live-out virutal registers
        self.set_def(func);
        self.visit(func);

        debug!(&self.block_data);

        // Compute program points
        self.compute_program_points(func);
    }

    pub fn compute_live_ranges(&mut self, func: &Function<T>, vreg: VReg) {
        let users = func.data.vreg_users.get(vreg);
        let mut def_pp = None;
        let mut use_pp = None;
        for user in users {
            let pp = self.inst_to_pp[&user.inst_id];

            if user.write {
                if def_pp.is_none() {
                    def_pp = Some(pp);
                    continue;
                }
                if let Some(def_pp) = &mut def_pp {
                    if &pp < def_pp {
                        *def_pp = pp
                    }
                    continue;
                }
            }

            if user.read {
                if use_pp.is_none() {
                    use_pp = Some(pp);
                    continue;
                }
                if let Some(use_pp) = &mut use_pp {
                    if *use_pp < pp {
                        *use_pp = pp
                    }
                    continue;
                }
            }
        }

        let lrs = LiveRanges(vec![LiveRange {
            start: def_pp.unwrap(),
            end: use_pp.unwrap(),
        }]);
        self.vreg_lrs_map.insert(vreg, lrs);
    }

    pub fn remove_vreg(&mut self, vreg: VReg) {
        self.remove_vreg_live_ranges(vreg);
        self.remove_vreg_from_block_data(vreg);
    }

    fn remove_vreg_from_block_data(&mut self, vreg: VReg) {
        for (_block_id, block_data) in &mut self.block_data {
            block_data.vreg_live_in.remove(&vreg);
            block_data.vreg_live_out.remove(&vreg);
        }
    }

    fn remove_vreg_live_ranges(&mut self, vreg: VReg) {
        self.vreg_lrs_map.remove(&vreg);
    }

    ////////

    pub fn compute_program_points(&mut self, func: &Function<T>) {
        let mut block_num = 0;
        for block_id in func.layout.block_iter() {
            let mut inst_num = 0u32;
            let mut local_vreg_lr_map = FxHashMap::default();
            let mut local_reg_lr_map = FxHashMap::default();

            // live-in
            for &live_in in &self.block_data[&block_id].vreg_live_in {
                local_vreg_lr_map.insert(
                    live_in,
                    LiveRange {
                        start: ProgramPoint(block_num, 0),
                        end: ProgramPoint(block_num, 0),
                    },
                );
            }
            for &live_in in &self.block_data[&block_id].reg_live_in {
                local_reg_lr_map.insert(
                    live_in,
                    LiveRanges(vec![LiveRange {
                        start: ProgramPoint(block_num, 0),
                        end: ProgramPoint(block_num, 0),
                    }]),
                );
            }

            inst_num += STEP;

            for inst_id in func.layout.inst_iter(block_id) {
                let inst = func.data.inst_ref(inst_id);

                self.inst_to_pp
                    .insert(inst_id, ProgramPoint(block_num, inst_num));

                // inputs
                for input in inst.data.input_vregs() {
                    local_vreg_lr_map.get_mut(&input).unwrap().end =
                        ProgramPoint(block_num, inst_num);
                    local_vreg_lr_map.get_mut(&input).unwrap().end =
                        ProgramPoint(block_num, inst_num);
                }
                for input in inst.data.input_regs() {
                    local_reg_lr_map
                        .get_mut(&T::RegInfo::to_reg_unit(input))
                        .unwrap()
                        .0
                        .last_mut()
                        .unwrap()
                        .end = ProgramPoint(block_num, inst_num);
                }

                // outputs
                for output in inst.data.output_vregs() {
                    local_vreg_lr_map
                        .entry(output)
                        .or_insert(LiveRange {
                            start: ProgramPoint(block_num, inst_num),
                            end: ProgramPoint(block_num, inst_num),
                        })
                        .end = ProgramPoint(block_num, inst_num);
                }
                for output in inst.data.output_regs() {
                    local_reg_lr_map
                        .entry(T::RegInfo::to_reg_unit(output))
                        .or_insert(LiveRanges(vec![]))
                        .0
                        .push(LiveRange {
                            start: ProgramPoint(block_num, inst_num),
                            end: ProgramPoint(block_num, inst_num),
                        })
                }

                inst_num += STEP;
            }

            // live-out
            for live_out in &self.block_data[&block_id].vreg_live_out {
                local_vreg_lr_map.get_mut(live_out).unwrap().end =
                    ProgramPoint(block_num, inst_num);
            }
            for live_out in &self.block_data[&block_id].reg_live_out {
                local_reg_lr_map
                    .get_mut(live_out)
                    .unwrap()
                    .0
                    .last_mut()
                    .unwrap()
                    .end = ProgramPoint(block_num, inst_num);
            }

            // merge local lr_map into lrs_map
            for (vreg, local_lr) in local_vreg_lr_map {
                self.vreg_lrs_map
                    .entry(vreg)
                    .or_insert(LiveRanges(vec![]))
                    .0
                    .push(local_lr)
            }
            for (reg, local_lr) in local_reg_lr_map {
                self.reg_lrs_map
                    .entry(reg)
                    .or_insert(LiveRanges(vec![]))
                    .0
                    .extend(local_lr.0.into_iter())
            }

            block_num += 1;
        }
    }

    ///////////

    // pub fn get_or_create_live_ranges(&mut self, vreg: VReg) -> &mut LiveRanges {
    //     self.lrs_map.entry(vreg).or_insert(LiveRanges(vec![]))
    // }

    ////////

    fn set_def(&mut self, func: &Function<T>) {
        for block_id in func.layout.block_iter() {
            self.block_data.entry(block_id).or_insert(BlockData::new());
            for inst_id in func.layout.inst_iter(block_id) {
                let inst = func.data.inst_ref(inst_id);
                self.set_def_on_inst(inst, block_id);
            }
        }
    }

    fn set_def_on_inst(
        &mut self,
        inst: &Instruction<<T::InstInfo as II>::Data>,
        block_id: BasicBlockId,
    ) {
        for output in inst.data.output_vregs() {
            self.block_data
                .entry(block_id)
                .or_insert_with(|| BlockData::new())
                .vreg_def
                .insert(output);
        }
        for output in inst.data.output_regs() {
            self.block_data
                .entry(block_id)
                .or_insert_with(|| BlockData::new())
                .reg_def
                .insert(T::RegInfo::to_reg_unit(output));
        }
    }

    fn visit(&mut self, func: &Function<T>) {
        for block_id in func.layout.block_iter() {
            for inst_id in func.layout.inst_iter(block_id) {
                let inst = func.data.inst_ref(inst_id);
                self.visit_inst(func, inst, block_id);
            }
        }
    }

    fn visit_inst(
        &mut self,
        func: &Function<T>,
        inst: &Instruction<<T::InstInfo as II>::Data>,
        block_id: BasicBlockId,
    ) {
        for input in inst.data.input_vregs() {
            self.propagate_vreg(func, input, block_id);
        }
        for input in inst.data.input_regs() {
            self.propagate_reg(func, T::RegInfo::to_reg_unit(input), block_id);
        }
    }

    fn propagate_vreg(&mut self, func: &Function<T>, input: VReg, block_id: BasicBlockId) {
        {
            let data = self.block_data.get_mut(&block_id).unwrap();

            if data.vreg_def.contains(&input) {
                return;
            }

            if !data.vreg_live_in.insert(input) {
                return;
            }
        }

        for pred_id in &func.data.basic_blocks[block_id].preds {
            if self
                .block_data
                .get_mut(pred_id)
                .unwrap()
                .vreg_live_out
                .insert(input)
            {
                self.propagate_vreg(func, input, *pred_id);
            }
        }
    }

    fn propagate_reg(&mut self, func: &Function<T>, input: RegUnit, block_id: BasicBlockId) {
        {
            let data = self.block_data.get_mut(&block_id).unwrap();

            if data.reg_def.contains(&input) {
                return;
            }

            if !data.reg_live_in.insert(input) {
                return;
            }
        }

        for pred_id in &func.data.basic_blocks[block_id].preds {
            if self
                .block_data
                .get_mut(pred_id)
                .unwrap()
                .reg_live_out
                .insert(input)
            {
                self.propagate_reg(func, input, *pred_id);
            }
        }
    }
}

impl LiveRanges {
    pub fn interfere(&self, other: &Self) -> bool {
        for x in &self.0 {
            for y in &other.0 {
                if x.interfere(y) {
                    return true;
                }
            }
        }
        false
    }

    pub fn interfere_with_single_range(&self, other: &LiveRange) -> bool {
        for x in &self.0 {
            if x.interfere(other) {
                return true;
            }
        }
        false
    }

    pub fn merge(&mut self, other: &Self) {
        if other.0.len() == 0 {
            return;
        }

        if self.0.len() == 0 {
            *self = other.clone();
            return;
        }

        let mut new = vec![];

        let mut z = vec![];
        let mut yi = 0;
        for x in &self.0 {
            while yi < other.0.len() {
                let y = &other.0[yi];
                if x.start.0 < y.start.0 {
                    new.push(y.clone());
                    break;
                }
                if x.start.0 == y.start.0 {
                    if x.interfere(y) {
                        new.push(LiveRange {
                            start: ::std::cmp::min(x.start, y.start),
                            end: ::std::cmp::max(x.end, y.end),
                        });
                    } else {
                        if x.start.1 < y.start.1 {
                            if x.end.1 == y.start.1 {
                                new.push(LiveRange {
                                    start: x.start,
                                    end: y.end,
                                });
                            } else {
                                new.push(x.clone());
                                new.push(y.clone())
                            }
                        } else {
                            if y.end.1 == x.start.1 {
                                new.push(LiveRange {
                                    start: y.start,
                                    end: x.end,
                                });
                            } else {
                                new.push(y.clone());
                                new.push(x.clone());
                            }
                        }
                    }
                    yi += 1;
                    break;
                }
                if x.start.0 > y.start.0 {
                    new.push(x.clone());
                    z.push(y.clone());
                    yi += 1;
                    continue;
                }
            }
        }

        for (i, z) in z.into_iter().enumerate() {
            new.insert(i, z)
        }
        if yi < other.0.len() - 1 {
            new.append(&mut other.0[yi..].to_vec())
        }

        self.0 = new;
    }
}

impl LiveRange {
    pub fn interfere(&self, other: &Self) -> bool {
        self.start < other.end && self.end > other.start
    }
}

impl BlockData {
    pub fn new() -> Self {
        BlockData {
            vreg_def: FxHashSet::default(),
            reg_def: FxHashSet::default(),
            vreg_live_in: FxHashSet::default(),
            vreg_live_out: FxHashSet::default(),
            reg_live_in: FxHashSet::default(),
            reg_live_out: FxHashSet::default(),
        }
    }
}
