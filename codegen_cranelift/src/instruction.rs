use super::LowerCtx;
use cranelift::{
    frontend::FunctionBuilder,
    prelude::{Block, InstBuilder, StackSlotData, StackSlotKind, Value},
};
use cranelift_codegen::ir::StackSlot;
use cranelift_module::Module;
use rustc_hash::FxHashMap;
use vicis_core::ir::{
    function::{
        basic_block::BasicBlockId,
        instruction::{Alloca, InstructionId, IntBinary, Load, Operand, Ret, Store},
        Function,
    },
    types::{Type as LlvmTy, TypeId},
    value::ValueId,
    value::{ConstantData, ConstantInt, Value as LlvmValue},
};

pub struct InstCompiler<'a, M: Module> {
    pub lower_ctx: &'a LowerCtx<'a, M>,
    pub llvm_func: &'a Function,
    pub builder: &'a mut FunctionBuilder<'a>,
    pub blocks: FxHashMap<BasicBlockId, Block>,
    pub insts: FxHashMap<InstructionId, Value>,
    pub stack_slots: FxHashMap<InstructionId, StackSlot>,
}

impl<'a, M: Module> InstCompiler<'a, M> {
    pub fn compile(&mut self, inst_id: InstructionId) {
        let inst = self.llvm_func.data.inst_ref(inst_id);

        match inst.operand {
            Operand::Alloca(Alloca { tys: [ty, _], .. }) => {
                assert!(*self.llvm_func.types.get(ty) == LlvmTy::Int(32));
                let slot = self.builder.create_stack_slot(StackSlotData::new(
                    StackSlotKind::ExplicitSlot,
                    4, /*i32*/
                ));
                self.stack_slots.insert(inst_id, slot);
            }
            Operand::Load(Load {
                addr, tys: [ty, _], ..
            }) => match self.value(addr, ty) {
                ValueKind::Value(_val) => todo!(),
                ValueKind::StackSlot(slot) => {
                    let dst =
                        self.builder
                            .ins()
                            .stack_load(self.lower_ctx.into_clif_ty(ty), slot, 0);
                    self.insts.insert(inst_id, dst);
                }
            },
            Operand::Store(Store {
                args: [src, dst],
                tys: [ty, _],
                ..
            }) => {
                let src = self.value(src, ty).as_value().expect("must be value");
                match self.value(dst, ty) {
                    ValueKind::Value(_val) => todo!(),
                    ValueKind::StackSlot(slot) => {
                        self.builder.ins().stack_store(src, slot, 0);
                    }
                }
            }
            Operand::IntBinary(IntBinary {
                ty,
                args: [lhs, rhs],
                ..
            }) => {
                let lhs = self.value(lhs, ty).as_value().expect("better use ? here");
                let rhs = self.value(rhs, ty).as_value().expect("better use ? here");
                let val = self.builder.ins().iadd(lhs, rhs);
                self.insts.insert(inst_id, val);
            }
            Operand::Ret(Ret { val: Some(val), ty }) => {
                let val = self.value(val, ty).as_value().expect("better use ? here");
                self.builder.ins().return_(&[val]);
            }
            _ => {}
        };
    }

    pub fn create_block_for(&mut self, block_id: BasicBlockId) -> Block {
        let block = self.builder.create_block();
        self.blocks.insert(block_id, block);
        block
    }

    fn value(&mut self, val_id: ValueId, ty: TypeId) -> ValueKind {
        match self.llvm_func.data.value_ref(val_id) {
            LlvmValue::Constant(ConstantData::Int(ConstantInt::Int32(i))) => ValueKind::Value(
                self.builder
                    .ins()
                    .iconst(self.lower_ctx.into_clif_ty(ty), *i as i64),
            ),
            LlvmValue::Argument(idx) => {
                let entry = self.llvm_func.layout.get_entry_block().unwrap();
                let entry = self.blocks[&entry];
                ValueKind::Value(self.builder.block_params(entry)[*idx])
            }
            LlvmValue::Instruction(inst_id) => {
                if let Some(val) = self.insts.get(inst_id).copied() {
                    ValueKind::Value(val)
                } else if let Some(slot) = self.stack_slots.get(inst_id).copied() {
                    ValueKind::StackSlot(slot)
                } else {
                    todo!()
                }
            }
            _ => todo!(),
        }
    }
}

enum ValueKind {
    Value(Value),
    StackSlot(StackSlot),
}

impl ValueKind {
    #[allow(dead_code)]
    #[inline(always)]
    fn as_value(&self) -> Option<Value> {
        match self {
            ValueKind::Value(v) => Some(*v),
            _ => None,
        }
    }

    #[allow(dead_code)]
    #[inline(always)]
    fn as_stack_slot(&self) -> Option<StackSlot> {
        match self {
            ValueKind::StackSlot(s) => Some(*s),
            _ => None,
        }
    }
}
