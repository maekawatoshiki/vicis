use super::LowerCtx;
use cranelift::{
    frontend::FunctionBuilder,
    prelude::{Block, InstBuilder, IntCC, StackSlotData, StackSlotKind, Value},
};
use cranelift_codegen::ir::StackSlot;
use cranelift_module::{FuncOrDataId, Module};
use rustc_hash::FxHashMap;
use vicis_core::ir::{
    function::{
        basic_block::BasicBlockId,
        instruction::{
            Alloca, Br, Call, CondBr, ICmp, ICmpCond, InstructionId, IntBinary, Load, Operand, Ret,
            Store,
        },
        Function,
    },
    module::name::Name,
    types as llvm_types,
    types::Type as LlvmTy,
    value::ValueId,
    value::{ConstantValue, Value as LlvmValue},
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
                assert!(ty.is_i32());
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
                _ => todo!(),
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
                    _ => todo!(),
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
            Operand::ICmp(ICmp { args, cond, ty }) => {
                let lhs = self
                    .value(args[0], ty)
                    .as_value()
                    .expect("better use ? here");
                let rhs = self
                    .value(args[1], ty)
                    .as_value()
                    .expect("better use ? here");
                let val = self.builder.ins().icmp(
                    match cond {
                        ICmpCond::Sle => IntCC::SignedLessThanOrEqual,
                        _ => todo!(),
                    },
                    lhs,
                    rhs,
                );
                self.insts.insert(inst_id, val);
            }
            Operand::Br(Br { block }) => {
                self.builder.ins().jump(self.blocks[&block], &[]); // TODO: Set block parameters.
            }
            Operand::CondBr(CondBr { arg, blocks }) => {
                let arg = self
                    .value(arg, llvm_types::I1)
                    .as_value()
                    .expect("better use ? here");
                self.builder.ins().brnz(arg, self.blocks[&blocks[0]], &[]); // TODO: Set block parameters.
                self.builder.ins().jump(self.blocks[&blocks[1]], &[]); // TODO: Set block parameters.
            }
            Operand::Call(Call {
                ref args, ref tys, ..
            }) => {
                let callee = self.value(args[0], llvm_types::VOID);
                let args = args[1..]
                    .iter()
                    .zip(&tys[1..])
                    .map(|(&arg, &ty)| self.value(arg, ty).as_value().unwrap())
                    .collect::<Vec<_>>();
                let name = callee
                    .as_global_name()
                    .expect("Only support calling named global function");
                let func_id = match self.lower_ctx.clif_mod.get_name(name.as_str()).unwrap() {
                    FuncOrDataId::Func(func_id) => func_id,
                    _ => todo!(),
                };
                let callee = self
                    .lower_ctx
                    .clif_mod
                    .declare_func_in_func(func_id, &mut self.builder.func);
                let call = self.builder.ins().call(callee, &args);
                let result_ty = tys[0];
                if !result_ty.is_void() {
                    let result = self.builder.inst_results(call)[0];
                    self.insts.insert(inst_id, result);
                }
            }
            Operand::Ret(Ret { val: Some(val), ty }) => {
                let val = self.value(val, ty).as_value().expect("better use ? here");
                self.builder.ins().return_(&[val]);
            }
            _ => {
                todo!()
            }
        };
    }

    pub fn create_block_for(&mut self, block_id: BasicBlockId) -> Block {
        if let Some(block) = self.blocks.get(&block_id) {
            return *block;
        }
        let block = self.builder.create_block();
        self.blocks.insert(block_id, block);
        block
    }

    fn value(&mut self, val_id: ValueId, ty: LlvmTy) -> ValueKind {
        match self.llvm_func.data.value_ref(val_id) {
            LlvmValue::Constant(ConstantValue::Int(i)) => ValueKind::Value(
                self.builder
                    .ins()
                    .iconst(self.lower_ctx.into_clif_ty(ty), i.cast_to_i64()),
            ),
            LlvmValue::Constant(ConstantValue::GlobalRef(Name::Name(name), _)) => {
                ValueKind::GlobalName(name.to_owned())
            }
            LlvmValue::Argument(arg) => {
                let entry = self.llvm_func.layout.get_entry_block().unwrap();
                let entry = self.blocks[&entry];
                ValueKind::Value(self.builder.block_params(entry)[arg.nth])
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
    GlobalName(String),
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

    #[allow(dead_code)]
    #[inline(always)]
    fn as_global_name(&self) -> Option<&String> {
        match self {
            ValueKind::GlobalName(s) => Some(s),
            _ => None,
        }
    }
}
