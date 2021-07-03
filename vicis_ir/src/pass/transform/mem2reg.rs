use crate::{
    ir::{
        function::{
            basic_block::BasicBlock,
            instruction::{Instruction, InstructionId, Opcode},
            Function,
        },
        value::Value,
    },
    pass::analysis::dom_tree,
};
use rustc_hash::FxHashMap;

pub struct Mem2Reg<'a> {
    func: &'a mut Function,
    dom_tree: dom_tree::DominatorTree<BasicBlock>,
    inst_indexes: InstructionIndexes,
}

type InstructionIndex = usize;

struct InstructionIndexes(FxHashMap<InstructionId, InstructionIndex>);

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
                dbg!(is_promotable);

                if !is_promotable {
                    continue;
                }

                let is_stored_only_once = self.is_stored_only_once(alloca);
                dbg!(is_stored_only_once);

                let is_only_used_in_single_block = self.is_only_used_in_single_block(alloca);
                dbg!(is_only_used_in_single_block);

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
            store_indexes: &Vec<(InstructionId, InstructionIndex)>,
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

    fn is_promotable(&self, alloca: &Instruction) -> bool {
        let alloca_id = alloca.id.unwrap();
        let alloca = alloca.operand.as_alloca().unwrap();
        let ty = alloca.ty();
        self.func.types.is_atomic(ty)
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
