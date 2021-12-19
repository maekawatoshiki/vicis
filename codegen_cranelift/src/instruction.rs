use super::LowerCtx;
use cranelift::{
    frontend::{FunctionBuilder, FunctionBuilderContext},
    prelude::{Block, InstBuilder, Value},
};
use cranelift_codegen::Context;
use cranelift_module::Module;
use rustc_hash::FxHashMap;
use vicis_core::ir::{
    function::{
        basic_block::BasicBlockId,
        instruction::{InstructionId, IntBinary, Operand, Ret},
        Function,
    },
    types::TypeId,
    value::ValueId,
    value::{ConstantData, ConstantInt, Value as LlvmValue},
};

pub fn compile_function_body<M: Module>(
    lower_ctx: &mut LowerCtx<'_, M>,
    builder_ctx: &mut FunctionBuilderContext,
    cl_ctx: &mut Context,
    llvm_func: &Function,
) {
    let mut builder = FunctionBuilder::new(&mut cl_ctx.func, builder_ctx);
    let mut compiler = InstCompiler {
        lower_ctx,
        llvm_func,
        builder: &mut builder,
        blocks: FxHashMap::default(),
        insts: FxHashMap::default(),
    };

    for (i, block_id) in llvm_func.layout.block_iter().enumerate() {
        let block = compiler.create_block_for(block_id);
        if i == 0 {
            compiler
                .builder
                .append_block_params_for_function_params(block);
        }
        compiler.builder.switch_to_block(block);
        compiler.builder.seal_block(block);
        for inst_id in llvm_func.layout.inst_iter(block_id) {
            compiler.compile(inst_id);
        }
    }

    compiler.builder.finalize();
}

struct InstCompiler<'a, M: Module> {
    lower_ctx: &'a LowerCtx<'a, M>,
    llvm_func: &'a Function,
    builder: &'a mut FunctionBuilder<'a>,
    blocks: FxHashMap<BasicBlockId, Block>,
    insts: FxHashMap<InstructionId, Value>,
}

impl<'a, M: Module> InstCompiler<'a, M> {
    fn compile(&mut self, inst_id: InstructionId) {
        let inst = self.llvm_func.data.inst_ref(inst_id);

        match inst.operand {
            Operand::IntBinary(IntBinary { ty, args, .. }) => {
                let lhs = self.value(args[0], ty);
                let rhs = self.value(args[1], ty);
                let val = self.builder.ins().iadd(lhs, rhs);
                self.insts.insert(inst_id, val);
            }
            Operand::Ret(Ret { val: Some(val), ty }) => {
                let val = self.value(val, ty);
                self.builder.ins().return_(&[val]);
            }
            _ => {}
        };
    }

    fn value(&mut self, val_id: ValueId, ty: TypeId) -> Value {
        match self.llvm_func.data.value_ref(val_id) {
            LlvmValue::Constant(ConstantData::Int(ConstantInt::Int32(i))) => self
                .builder
                .ins()
                .iconst(self.lower_ctx.into_clif_ty(ty), *i as i64),
            LlvmValue::Argument(idx) => {
                let entry = self.llvm_func.layout.get_entry_block().unwrap();
                let entry = self.blocks[&entry];
                self.builder.block_params(entry)[*idx]
            }
            LlvmValue::Instruction(inst_id) => self.insts[inst_id],
            _ => todo!(),
        }
    }

    fn create_block_for(&mut self, block_id: BasicBlockId) -> Block {
        let block = self.builder.create_block();
        self.blocks.insert(block_id, block);
        block
    }
}
