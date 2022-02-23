use crate::{
    ir::{
        function::{
            basic_block::{BasicBlock, BasicBlockId},
            instruction::{Instruction, InstructionId, Opcode, Operand, Phi},
            Function,
        },
        value::{Value, ValueId},
    },
    pass::{analysis::dom_tree, transform::sccp::SCCP, TransformPass},
};
use rustc_hash::{FxHashMap, FxHashSet};
use std::{any::Any, cmp::Ordering, collections::BinaryHeap};

pub struct Mem2RegPass;

pub struct Mem2Reg<'a> {
    func: &'a mut Function,
    dom_tree: dom_tree::DominatorTree<BasicBlock>,
    inst_indexes: InstructionIndexes,
}

type InstructionIndex = usize;

struct InstructionIndexes(FxHashMap<InstructionId, InstructionIndex>);

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct BlockLevel(usize, BasicBlockId);

struct RenameData {
    cur: BasicBlockId,
    pred: Option<BasicBlockId>,
    incoming: FxHashMap<InstructionId, ValueId>,
}

impl<'a> Mem2Reg<'a> {
    pub fn new(func: &'a mut Function) -> Self {
        Self {
            dom_tree: dom_tree::DominatorTree::new(func),
            inst_indexes: InstructionIndexes::default(),
            func,
        }
    }

    pub fn run(&mut self) {
        let mut single_store_alloca_list = vec![];
        let mut single_block_alloca_list = vec![];
        let mut multi_block_alloca_list = vec![];

        for block_id in self.func.layout.block_iter() {
            for inst_id in self.func.layout.inst_iter(block_id) {
                let inst = self.func.data.inst_ref(inst_id);
                if !inst.opcode.is_alloca() {
                    continue;
                }

                let alloca = inst;

                let is_promotable = self.is_promotable(alloca);
                log::debug!("is_promotable = {is_promotable}");

                if !is_promotable {
                    continue;
                }

                let is_stored_only_once = self.is_stored_only_once(alloca);
                log::debug!("is_stored_only_once = {is_stored_only_once}");

                let is_only_used_in_single_block = self.is_only_used_in_single_block(alloca);
                log::debug!("is_only_used_in_single_block = {is_only_used_in_single_block}");

                if is_stored_only_once {
                    single_store_alloca_list.push(inst_id);
                    continue;
                }

                if is_only_used_in_single_block {
                    single_block_alloca_list.push(inst_id);
                    continue;
                }

                multi_block_alloca_list.push(inst_id);
            }
        }

        for alloca in single_store_alloca_list {
            self.promote_single_store_alloca(alloca);
        }

        for alloca in single_block_alloca_list {
            self.promote_single_block_alloca(alloca);
        }

        let mut phi_to_alloca = FxHashMap::default();
        let mut added_phis = FxHashMap::default();
        for &alloca in &multi_block_alloca_list {
            self.promote_multi_block_alloca(alloca, &mut phi_to_alloca, &mut added_phis);
        }

        self.rename(multi_block_alloca_list, phi_to_alloca, added_phis);

        SCCP::new(self.func).run();
    }

    fn promote_single_store_alloca(&mut self, alloca_id: InstructionId) {
        let mut src = None;
        let mut store_to_remove = None;
        let mut loads_to_remove = vec![];

        for &user_id in self.func.data.users_of(alloca_id) {
            let user = self.func.data.inst_ref(user_id);
            match user.opcode {
                Opcode::Load => loads_to_remove.push(user_id),
                Opcode::Store => {
                    src = Some(user.operand.as_store().unwrap().src_val());
                    store_to_remove = Some(user_id);
                }
                _ => unreachable!(),
            }
        }

        let src = src.unwrap();
        let store_to_remove = store_to_remove.unwrap();
        let store_idx = self.inst_indexes.get(self.func, store_to_remove);

        let mut remove_all_loads = true;
        loads_to_remove.retain(|&load_id| {
            let load = self.func.data.inst_ref(load_id);
            let store = self.func.data.inst_ref(store_to_remove);
            let valid = if load.parent == store.parent {
                let load_idx = self.inst_indexes.get(self.func, load_id);
                store_idx < load_idx
            } else {
                self.dom_tree.dominates(store.parent, load.parent)
            };
            remove_all_loads &= valid;
            valid
        });

        if remove_all_loads {
            self.func.remove_inst(store_to_remove);
            self.func.remove_inst(alloca_id);
        }

        for load_id in loads_to_remove {
            self.func.remove_inst(load_id);
            for user_id in self.func.data.users_of(load_id).clone() {
                self.func.data.replace_inst_arg(user_id, load_id, src);
            }
        }
    }

    fn promote_single_block_alloca(&mut self, alloca_id: InstructionId) {
        fn find_nearest_store(
            store_indexes: &[(InstructionId, InstructionIndex)],
            load_idx: InstructionIndex,
        ) -> Option<InstructionId> {
            let i = store_indexes
                .binary_search_by(|(_, store_idx)| store_idx.cmp(&load_idx))
                .unwrap_or_else(|x| x);
            if i == 0 {
                return None;
            }
            Some(store_indexes[i - 1].0)
        }

        let mut store_indexes = vec![];
        let mut loads = vec![];

        for &user_id in self.func.data.users_of(alloca_id) {
            let user = self.func.data.inst_ref(user_id);
            match user.opcode {
                Opcode::Store => {
                    store_indexes.push((user_id, self.inst_indexes.get(self.func, user_id)))
                }
                Opcode::Load => loads.push(user_id),
                _ => unreachable!(),
            }
        }

        store_indexes.sort_by(|(_, x), (_, y)| x.cmp(y));

        let mut remove_all_access = true;
        let mut stores_to_remove = vec![];

        for load_id in loads {
            let load_idx = self.inst_indexes.get(self.func, load_id);
            let nearest_store_id = match find_nearest_store(&store_indexes, load_idx) {
                Some(nearest_store_id) => nearest_store_id,
                None => {
                    remove_all_access = false;
                    continue;
                }
            };
            let nearest_store = self.func.data.inst_ref(nearest_store_id);
            let src = nearest_store.operand.as_store().unwrap().src_val();

            stores_to_remove.push(nearest_store_id);

            self.func.remove_inst(load_id);
            for user_id in self.func.data.users_of(load_id).clone() {
                self.func.data.replace_inst_arg(user_id, load_id, src);
            }
        }

        if remove_all_access {
            self.func.remove_inst(alloca_id);
        }

        for store in stores_to_remove {
            self.func.remove_inst(store);
        }
    }

    fn promote_multi_block_alloca(
        &mut self,
        alloca_id: InstructionId,
        phi_to_alloca: &mut FxHashMap<InstructionId, InstructionId>,
        added_phis: &mut FxHashMap<BasicBlockId, Vec<InstructionId>>,
    ) {
        let mut def_blocks = vec![];
        let mut use_blocks = vec![];
        let mut livein_blocks = FxHashSet::default();

        for &user_id in self.func.data.users_of(alloca_id) {
            let user = self.func.data.inst_ref(user_id);
            match user.opcode {
                Opcode::Store => def_blocks.push(user.parent),
                Opcode::Load => use_blocks.push(user.parent),
                _ => unreachable!(),
            }
        }

        let mut worklist = use_blocks;
        while let Some(block) = worklist.pop() {
            if !livein_blocks.insert(block) {
                continue;
            }
            for pred in self.func.data.basic_blocks[block].preds() {
                if def_blocks.contains(pred) {
                    continue;
                }
                worklist.push(*pred)
            }
        }

        let mut queue = def_blocks
            .iter()
            .map(|&def| BlockLevel(self.dom_tree.level_of(def).unwrap(), def))
            .collect::<BinaryHeap<_>>();
        let mut visited_worklist = FxHashSet::default();
        let mut visited_queue = FxHashSet::default();

        while let Some(BlockLevel(root_level, root_block_id)) = queue.pop() {
            let mut worklist = vec![root_block_id];
            visited_worklist.insert(root_block_id);

            while let Some(block_id) = worklist.pop() {
                let block = &self.func.data.basic_blocks[block_id];
                for succ_id in block.succs().clone() {
                    let succ_level = self.dom_tree.level_of(succ_id).unwrap();
                    if succ_level > root_level {
                        continue;
                    }
                    if !visited_queue.insert(succ_id) {
                        continue;
                    }
                    if !livein_blocks.contains(&succ_id) {
                        continue;
                    }

                    {
                        let ty = self
                            .func
                            .data
                            .inst_ref(alloca_id)
                            .operand
                            .as_alloca()
                            .unwrap()
                            .ty();
                        let phi = Opcode::Phi
                            .with_block(succ_id)
                            .with_operand(Operand::Phi(Phi {
                                ty,
                                args: vec![],
                                blocks: vec![],
                            }));
                        let phi_id = self.func.data.create_inst(phi);
                        self.func.layout.insert_inst_at_start(phi_id, succ_id);
                        added_phis
                            .entry(succ_id)
                            .or_insert_with(Vec::new)
                            .push(phi_id);
                        phi_to_alloca.insert(phi_id, alloca_id);
                    }

                    if !def_blocks.contains(&succ_id) {
                        queue.push(BlockLevel(succ_level, succ_id));
                    }
                }

                if let Some(dom_children) = self.dom_tree.children_of(block_id) {
                    for child in dom_children {
                        if visited_worklist.insert(*child) {
                            worklist.push(*child);
                        }
                    }
                }
            }
        }
    }

    fn rename(
        &mut self,
        alloca_list: Vec<InstructionId>,
        phi_to_alloca: FxHashMap<InstructionId, InstructionId>,
        mut added_phis: FxHashMap<BasicBlockId, Vec<InstructionId>>,
    ) {
        let entry = self.func.layout.first_block.unwrap();

        let mut visited = FxHashSet::default();
        let mut worklist = vec![RenameData {
            cur: entry,
            pred: None,
            incoming: FxHashMap::default(),
        }];

        while let Some(data) = worklist.pop() {
            self.rename_sub(
                &alloca_list,
                &phi_to_alloca,
                &mut worklist,
                &mut added_phis,
                &mut visited,
                data,
            );
        }

        for alloca_id in alloca_list {
            self.func.remove_inst(alloca_id);
        }
    }

    fn rename_sub(
        &mut self,
        alloca_list: &[InstructionId],
        phi_to_alloca: &FxHashMap<InstructionId, InstructionId>,
        worklist: &mut Vec<RenameData>,
        added_phis: &mut FxHashMap<BasicBlockId, Vec<InstructionId>>,
        visited: &mut FxHashSet<BasicBlockId>,
        mut data: RenameData,
    ) {
        loop {
            for phi_id in added_phis.get(&data.cur).unwrap_or(&vec![]) {
                let alloca_id = phi_to_alloca[phi_id];
                let incoming_id = data
                    .incoming
                    .get_mut(&alloca_id)
                    .expect("TODO: return undef");
                let phi = self.func.data.inst_ref_mut(*phi_id);
                let phi = phi.operand.as_phi_mut().unwrap();
                phi.args_mut().push(*incoming_id);
                phi.blocks_mut().push(data.pred.unwrap());
                self.func.data.validate_inst_uses(*phi_id);
                *incoming_id = self.func.data.create_value(Value::Instruction(*phi_id));
            }

            if !visited.insert(data.cur) {
                break;
            }

            let mut removal_list = vec![];

            for inst_id in self.func.layout.inst_iter(data.cur) {
                let inst = self.func.data.inst_ref(inst_id);
                let alloca_id = *self
                    .func
                    .data
                    .value_ref(match inst.opcode {
                        Opcode::Store => inst.operand.as_store().unwrap().dst_val(),
                        Opcode::Load => inst.operand.as_load().unwrap().src_val(),
                        _ => continue,
                    })
                    .as_inst()
                    .unwrap();
                if !alloca_list.contains(&alloca_id) {
                    continue;
                }
                match inst.opcode {
                    Opcode::Store => {
                        data.incoming
                            .insert(alloca_id, inst.operand.as_store().unwrap().src_val());
                    }
                    Opcode::Load => {
                        if let Some(val) = data.incoming.get(&alloca_id) {
                            self.func.data.replace_all_uses(inst_id, *val);
                        }
                    }
                    _ => unreachable!(),
                }
                removal_list.push(inst_id);
            }

            for remove in removal_list {
                self.func.remove_inst(remove);
            }

            let block = &self.func.data.basic_blocks[data.cur];

            if block.succs().is_empty() {
                break;
            }

            data.pred = Some(data.cur);
            let mut succ_iter = block.succs().iter();
            data.cur = *succ_iter.next().unwrap();
            for succ in succ_iter {
                worklist.push(RenameData {
                    cur: *succ,
                    pred: data.pred,
                    incoming: data.incoming.clone(),
                })
            }
        }
    }

    fn is_promotable(&self, alloca: &Instruction) -> bool {
        let alloca_id = alloca.id.unwrap();
        let alloca = alloca.operand.as_alloca().unwrap();
        let ty = alloca.ty();
        (ty.is_primitive() || ty.is_pointer(&self.func.types))
            && self.func.data.users_of(alloca_id).iter().all(|&user_id| {
                let user = self.func.data.inst_ref(user_id);
                user.opcode.is_load()
                    || (user.opcode.is_store() && {
                        let dst_id = user.operand.as_store().unwrap().dst_val();
                        let dst = self.func.data.value_ref(dst_id);
                        matches!(dst, Value::Instruction(id) if id == &alloca_id)
                    })
            })
    }

    fn is_stored_only_once(&self, alloca: &Instruction) -> bool {
        let alloca_id = alloca.id.unwrap();
        self.func
            .data
            .users_of(alloca_id)
            .iter()
            .fold(0usize, |acc, &user_id| {
                let user = self.func.data.inst_ref(user_id);
                user.opcode.is_store() as usize + acc
            })
            == 1
    }

    fn is_only_used_in_single_block(&self, alloca: &Instruction) -> bool {
        let alloca_id = alloca.id.unwrap();
        let mut last_parent = None;
        self.func.data.users_of(alloca_id).iter().all(|&user_id| {
            let user = self.func.data.inst_ref(user_id);
            let eq = last_parent.get_or_insert(user.parent) == &user.parent;
            last_parent = Some(user.parent);
            eq
        })
    }
}

impl Default for InstructionIndexes {
    fn default() -> Self {
        Self(FxHashMap::default())
    }
}

impl InstructionIndexes {
    pub fn get(&mut self, func: &Function, inst_id: InstructionId) -> InstructionIndex {
        if let Some(idx) = self.0.get(&inst_id) {
            return *idx;
        }

        let inst = func.data.inst_ref(inst_id);
        for (i, inst_id) in func.layout.inst_iter(inst.parent).enumerate() {
            let opcode = func.data.inst_ref(inst_id).opcode;
            let is_interesting = opcode.is_store() || opcode.is_load() || opcode.is_alloca();
            if is_interesting {
                self.0.insert(inst_id, i);
            }
        }

        self.get(func, inst_id)
    }
}

impl TransformPass<Function> for Mem2RegPass {
    fn run_on(&self, func: &mut Function, _result: &mut Box<dyn Any>) {
        Mem2Reg::new(func).run();
    }
}

impl Ord for BlockLevel {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for BlockLevel {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
