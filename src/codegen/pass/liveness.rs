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
    pub vreg_lrs_map: FxHashMap<VReg, LiveRange>,
    pub reg_lrs_map: FxHashMap<RegUnit, LiveRange>,
    pub inst_to_pp: FxHashMap<InstructionId<<T::InstInfo as II>::Data>, ProgramPoint>,
}

// `LiveSegment`s are sorted in ascending order by `start`
#[derive(Debug, Clone)]
pub struct LiveRange(pub Vec<LiveSegment>);

#[derive(Debug, Clone)]
pub struct LiveSegment {
    pub start: ProgramPoint,
    pub end: ProgramPoint,
}

#[derive(Debug)]
pub struct BlockData {
    def: FxHashSet<Reg>,
    live_in: FxHashSet<Reg>,
    live_out: FxHashSet<Reg>,
}

#[derive(Debug, PartialEq, Hash, Eq, Copy, Clone)]
enum Reg {
    Phys(RegUnit),
    Virt(VReg),
}

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

impl<T: TargetIsa> Liveness<T> {
    pub fn new() -> Self {
        Self {
            block_data: FxHashMap::default(),
            reg_lrs_map: FxHashMap::default(),
            vreg_lrs_map: FxHashMap::default(),
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

        let lrs = LiveRange(vec![LiveSegment {
            start: def_pp.unwrap(),
            end: use_pp.unwrap(),
        }]);
        self.vreg_lrs_map.insert(vreg, lrs);
    }

    pub fn vreg_range(&self, vreg: &VReg) -> Option<&LiveRange> {
        self.vreg_lrs_map.get(vreg)
    }

    pub fn remove_vreg(&mut self, vreg: VReg) {
        self.remove_vreg_live_ranges(vreg);
        self.remove_vreg_from_block_data(vreg);
    }

    fn remove_vreg_from_block_data(&mut self, vreg: VReg) {
        for (_block_id, block_data) in &mut self.block_data {
            block_data.live_in.remove(&Reg::Virt(vreg));
            block_data.live_out.remove(&Reg::Virt(vreg));
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
            for &live_in in &self.block_data[&block_id].live_in {
                match live_in {
                    Reg::Virt(live_in) => {
                        local_vreg_lr_map.insert(
                            live_in,
                            LiveSegment {
                                start: ProgramPoint(block_num, 0),
                                end: ProgramPoint(block_num, 0),
                            },
                        );
                    }
                    Reg::Phys(live_in) => {
                        local_reg_lr_map.insert(
                            live_in,
                            LiveRange(vec![LiveSegment {
                                start: ProgramPoint(block_num, 0),
                                end: ProgramPoint(block_num, 0),
                            }]),
                        );
                    }
                }
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
                        .or_insert(LiveSegment {
                            start: ProgramPoint(block_num, inst_num),
                            end: ProgramPoint(block_num, inst_num),
                        })
                        .end = ProgramPoint(block_num, inst_num);
                }
                for output in inst.data.output_regs() {
                    local_reg_lr_map
                        .entry(T::RegInfo::to_reg_unit(output))
                        .or_insert(LiveRange(vec![]))
                        .0
                        .push(LiveSegment {
                            start: ProgramPoint(block_num, inst_num),
                            end: ProgramPoint(block_num, inst_num),
                        })
                }

                inst_num += STEP;
            }

            // live-out
            for live_out in &self.block_data[&block_id].live_out {
                match live_out {
                    Reg::Virt(live_out) => {
                        local_vreg_lr_map.get_mut(live_out).unwrap().end =
                            ProgramPoint(block_num, inst_num);
                    }
                    Reg::Phys(live_out) => {
                        local_reg_lr_map
                            .get_mut(live_out)
                            .unwrap()
                            .0
                            .last_mut()
                            .unwrap()
                            .end = ProgramPoint(block_num, inst_num);
                    }
                }
            }

            // merge local lr_map into lrs_map
            for (vreg, local_lr) in local_vreg_lr_map {
                self.vreg_lrs_map
                    .entry(vreg)
                    .or_insert(LiveRange(vec![]))
                    .0
                    .push(local_lr)
            }
            for (reg, local_lr) in local_reg_lr_map {
                self.reg_lrs_map
                    .entry(reg)
                    .or_insert(LiveRange(vec![]))
                    .0
                    .extend(local_lr.0.into_iter())
            }

            block_num += 1;
        }
    }

    ///////////

    // pub fn get_or_create_live_ranges(&mut self, vreg: VReg) -> &mut LiveRange {
    //     self.lrs_map.entry(vreg).or_insert(LiveRange(vec![]))
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
                .def
                .insert(Reg::Virt(output));
        }
        for output in inst.data.output_regs() {
            self.block_data
                .entry(block_id)
                .or_insert_with(|| BlockData::new())
                .def
                .insert(Reg::Phys(T::RegInfo::to_reg_unit(output)));
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
            self.propagate_reg(func, Reg::Virt(input), block_id);
        }
        for input in inst.data.input_regs() {
            self.propagate_reg(func, Reg::Phys(T::RegInfo::to_reg_unit(input)), block_id);
        }
    }

    fn propagate_reg(&mut self, func: &Function<T>, input: Reg, block_id: BasicBlockId) {
        {
            let data = self.block_data.get_mut(&block_id).unwrap();

            if data.def.contains(&input) {
                return;
            }

            if !data.live_in.insert(input) {
                return;
            }
        }

        for pred_id in &func.data.basic_blocks[block_id].preds {
            if self
                .block_data
                .get_mut(pred_id)
                .unwrap()
                .live_out
                .insert(input)
            {
                self.propagate_reg(func, input, *pred_id);
            }
        }
    }
}

impl LiveRange {
    pub fn first_seg(&self) -> Option<&LiveSegment> {
        self.0.get(0)
    }

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

    pub fn interfere_with_segment(&self, other: &LiveSegment) -> bool {
        for x in &self.0 {
            if x.interfere(other) {
                return true;
            }
        }
        false
    }

    // Assume self doesn't interfere with other
    pub fn merge(&mut self, other: &Self) {
        self.0.append(&mut other.0.clone());
        self.0.sort_by(|x, y| x.start.cmp(&y.start));
    }
}

impl LiveSegment {
    pub fn new_point(pp: ProgramPoint) -> Self {
        Self { start: pp, end: pp }
    }

    pub fn interfere(&self, other: &Self) -> bool {
        self.start < other.end && self.end > other.start
    }
}

impl BlockData {
    pub fn new() -> Self {
        BlockData {
            def: FxHashSet::default(),
            live_in: FxHashSet::default(),
            live_out: FxHashSet::default(),
        }
    }
}
