use crate::codegen::{
    basic_block::BasicBlockId,
    function::Function,
    instruction::{Instruction, InstructionData},
    module::Module,
    register::{Reg, VReg},
    target::Target,
};
use rustc_hash::{FxHashMap, FxHashSet};
use std::cmp::Ordering;

pub struct Liveness {
    pub block_data: FxHashMap<BasicBlockId, BlockData>,
    pub vreg_lrs_map: FxHashMap<VReg, LiveRanges>,
    pub reg_lrs_map: FxHashMap<Reg, LiveRanges>,
    // pp_arena: Arena<ProgramPoint>,
}

#[derive(Debug)]
pub struct LiveRanges(pub Vec<LiveRange>);

#[derive(Debug)]
pub struct LiveRange {
    pub start: ProgramPoint,
    pub end: ProgramPoint,
}

#[derive(Debug)]
pub struct BlockData {
    def: FxHashSet<VReg>,
    live_in: FxHashSet<VReg>,
    live_out: FxHashSet<VReg>,
}

// pub type ProgramPointId = Id<ProgramPoint>;

#[derive(Debug, Clone, Copy)]
pub struct ProgramPoint(pub u32, pub u32);

impl PartialOrd for ProgramPoint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.0 < other.1 {
            return Some(Ordering::Less);
        }

        assert_eq!(self.0, other.1);

        if self.0 < other.1 {
            return Some(Ordering::Less);
        }

        if self.0 < other.1 {
            return Some(Ordering::Equal);
        }

        assert!(self.0 > other.1);

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

// pub fn run_on_module<T: Target>(module: &mut Module<T>) {
//     for (_, func) in &mut module.functions {
//         run_on_function(func);
//     }
// }

// pub fn run_on_function<T: Target>(function: &mut Function<T>) -> Liveness {
//     // for block_id in function.layout.block_iter() {
//     //     for inst_id in function.layout.inst_iter(block_id) {
//     //         let inst = function.data.inst_ref(inst_id);
//     //     }
//     // }
//     todo!()
// }

impl Liveness {
    pub fn new() -> Self {
        Self {
            block_data: FxHashMap::default(),
            vreg_lrs_map: FxHashMap::default(),
            reg_lrs_map: FxHashMap::default(),
        }
    }

    pub fn analyze_function<T: Target>(&mut self, func: &Function<T>) {
        // Analyze live-in and live-out virutal registers
        self.set_def(func);
        self.visit(func);

        // Compute program points
        self.compute_program_points(func);
    }

    ////////

    pub fn compute_program_points<T: Target>(&mut self, func: &Function<T>) {
        let mut block_num = 0;
        for block_id in func.layout.block_iter() {
            const STEP: u32 = 16;
            let mut inst_num = 0u32;
            let mut local_lr_map = FxHashMap::default();

            // live-in
            for &live_in in &self.block_data[&block_id].live_in {
                local_lr_map.insert(
                    live_in,
                    LiveRange {
                        start: ProgramPoint(block_num, 0),
                        end: ProgramPoint(block_num, 0),
                    },
                );
            }

            inst_num += STEP;

            for inst_id in func.layout.inst_iter(block_id) {
                let inst = func.data.inst_ref(inst_id);

                // inputs
                for input in inst.data.input_vregs() {
                    local_lr_map.get_mut(&input).unwrap().end = ProgramPoint(block_num, inst_num);
                }

                // outputs
                for output in inst.data.output_vregs() {
                    local_lr_map
                        .entry(output)
                        .or_insert(LiveRange {
                            start: ProgramPoint(block_num, inst_num),
                            end: ProgramPoint(block_num, inst_num),
                        })
                        .end = ProgramPoint(block_num, inst_num);
                }

                inst_num += STEP;
            }

            // live-out
            for live_out in &self.block_data[&block_id].live_out {
                local_lr_map.get_mut(live_out).unwrap().end = ProgramPoint(block_num, inst_num);
            }

            // merge local_lr_map into lrs_map
            for (vreg, local_lr) in local_lr_map {
                self.vreg_lrs_map
                    .entry(vreg)
                    .or_insert(LiveRanges(vec![]))
                    .0
                    .push(local_lr)
            }

            block_num += 1;
        }
    }

    ///////////

    // pub fn get_or_create_live_ranges(&mut self, vreg: VReg) -> &mut LiveRanges {
    //     self.lrs_map.entry(vreg).or_insert(LiveRanges(vec![]))
    // }

    ////////

    fn set_def<T: Target>(&mut self, func: &Function<T>) {
        for block_id in func.layout.block_iter() {
            for inst_id in func.layout.inst_iter(block_id) {
                let inst = func.data.inst_ref(inst_id);
                self.set_def_on_inst(inst, block_id);
            }
        }
    }

    fn set_def_on_inst<InstData: InstructionData>(
        &mut self,
        inst: &Instruction<InstData>,
        block_id: BasicBlockId,
    ) {
        let add_def = |block_data: &mut FxHashMap<BasicBlockId, BlockData>, r: VReg| {
            block_data
                .entry(block_id)
                .or_insert_with(|| BlockData {
                    def: FxHashSet::default(),
                    live_in: FxHashSet::default(),
                    live_out: FxHashSet::default(),
                })
                .def
                .insert(r);
        };
        for output in inst.data.output_vregs() {
            add_def(&mut self.block_data, output);
        }
    }

    fn visit<T: Target>(&mut self, func: &Function<T>) {
        for block_id in func.layout.block_iter() {
            for inst_id in func.layout.inst_iter(block_id) {
                let inst = func.data.inst_ref(inst_id);
                self.visit_inst(func, inst, block_id);
            }
        }
    }

    fn visit_inst<T: Target>(
        &mut self,
        func: &Function<T>,
        inst: &Instruction<T::InstData>,
        block_id: BasicBlockId,
    ) {
        for input in inst.data.input_vregs() {
            self.propagate(func, input, block_id);
        }
    }

    fn propagate<T: Target>(&mut self, func: &Function<T>, input: VReg, block_id: BasicBlockId) {
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
                self.propagate(func, input, *pred_id);
            }
        }
    }

    pub fn interfere_live_ranges(&self, x: &LiveRanges, y: &LiveRanges) -> bool {
        // let x_start_pp = self.pp_arena[x.start];
        // let x_end_pp = self.pp_arena[x.end];
        // let y_start_pp = self.pp_arena[y.start];
        // let y_end_pp = self.pp_arena[y.end];
        false
    }
}
