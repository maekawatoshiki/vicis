// Sparse Conditional Constant Propagation

use crate::ir::{
    function::{
        basic_block::BasicBlockId,
        instruction::{Br, Instruction, Opcode, Operand},
        Function,
    },
    value::{ConstantValue, Value},
};
use std::collections::VecDeque;

pub struct SCCP<'a> {
    func: &'a mut Function,
}

impl<'a> SCCP<'a> {
    pub fn new(func: &'a mut Function) -> Self {
        Self { func }
    }

    pub fn run(&mut self) {
        let mut foldable = VecDeque::new();
        let mut foldable_condbr = VecDeque::new();
        let mut changed = false;

        for block_id in self.func.layout.block_iter() {
            for inst_id in self.func.layout.inst_iter(block_id) {
                let inst = self.func.data.inst_ref(inst_id);

                if self.is_foldable(inst) {
                    foldable.push_back(inst_id);
                }

                if self.is_foldable_condbr(inst) {
                    foldable_condbr.push_back(inst_id);
                }
            }
        }

        while let Some(inst_id) = foldable_condbr.pop_front() {
            let condbr = &self.func.data.inst_ref(inst_id);
            let block = condbr.parent;
            let condbr = condbr.operand.as_condbr().unwrap();
            let [dst, not_dst] = condbr.blocks as [BasicBlockId; 2];
            // Create a new Br instruction to replace CondBr with.
            let br = Opcode::Br
                .with_block(block)
                .with_operand(Operand::Br(Br { block: dst }));
            let br = self.func.data.create_inst(br);
            // Append Br and remove CondBr
            self.func.layout.append_inst(br, block);
            self.func.remove_inst(inst_id);
            // Remove successor and predecessor from blocks properly
            self.func.data.remove_block_succ(block, not_dst);
            self.func.data.remove_block_pred(not_dst, block);
            changed |= true;
        }

        if changed {
            self.fold_phi();
        }

        while let Some(inst_id) = foldable.pop_front() {
            let inst = &self.func.data.inst_ref(inst_id);
            let folded = match inst.fold_consts(&self.func.data) {
                Some(folded) => folded,
                None => continue,
            };
            let folded = self.func.data.create_value(Value::Constant(folded));
            self.func.data.replace_all_uses(inst_id, folded);
            self.func.remove_inst(inst_id);
            changed |= true;
        }

        if changed {
            self.run()
        }
    }

    fn is_foldable(&self, inst: &Instruction) -> bool {
        matches!(
            inst.opcode,
            Opcode::Add | Opcode::Sub | Opcode::Mul | Opcode::ICmp | Opcode::Zext | Opcode::Sext
        ) && inst
            .operand
            .args()
            .iter()
            .map(|arg| self.func.data.value_ref(*arg))
            .all(|arg| matches!(arg, Value::Constant(ConstantValue::Int(_))))
    }

    fn is_foldable_condbr(&self, inst: &Instruction) -> bool {
        matches!(inst.opcode, Opcode::CondBr)
            && inst
                .operand
                .args()
                .iter()
                .map(|arg| self.func.data.value_ref(*arg))
                .all(|arg| matches!(arg, Value::Constant(ConstantValue::Int(_))))
    }

    fn fold_phi(&mut self) {
        let mut remove_list = vec![];

        for block_id in self.func.layout.block_iter() {
            for inst_id in self.func.layout.inst_iter(block_id) {
                let inst = self.func.data.inst_ref(inst_id);

                if inst.opcode != Opcode::Phi {
                    continue;
                }

                let phi = inst;
                let phi = phi.operand.as_phi().unwrap();

                // TODO: Now very limited case supported
                let mut arg = None;
                let phi_has_single_valid_arg = phi.blocks.iter().zip(phi.args.iter()).fold(
                    0usize,
                    |acc, (&block_id, &arg_id)| {
                        let not_empty = !self.func.data.basic_blocks[block_id].preds().is_empty();
                        if not_empty {
                            arg = Some(arg_id);
                        }
                        acc + not_empty as usize
                    },
                ) == 1;

                if !phi_has_single_valid_arg {
                    continue;
                }

                self.func.data.replace_all_uses(inst_id, arg.unwrap());
                remove_list.push(inst_id)
            }
        }

        for id in remove_list {
            self.func.remove_inst(id);
        }
    }
}
