use super::Modules;
use cranelift::{
    frontend::{FunctionBuilder, FunctionBuilderContext},
    prelude::{InstBuilder, Value},
};
use cranelift_codegen::Context;
use cranelift_module::Module;
use vicis_core::ir::{
    function::{
        instruction::{InstructionId, Operand, Ret},
        Function,
    },
    types::TypeId,
    value::ValueId,
    value::{ConstantData, ConstantInt, Value as LlvmValue},
};

pub fn compile_function_body<M: Module>(
    modules: &mut Modules<'_, M>,
    builder_ctx: &mut FunctionBuilderContext,
    cl_ctx: &mut Context,
    llvm_func: &Function,
) {
    let mut builder = FunctionBuilder::new(&mut cl_ctx.func, builder_ctx);

    for (i, block_id) in llvm_func.layout.block_iter().enumerate() {
        let block = builder.create_block();
        if i == 0 {
            builder.append_block_params_for_function_params(block);
        }
        builder.switch_to_block(block);
        builder.seal_block(block);
        for inst_id in llvm_func.layout.inst_iter(block_id) {
            compile_instruction(modules, llvm_func, &mut builder, inst_id);
        }
    }

    builder.finalize();
}

fn compile_instruction<M: Module>(
    modules: &Modules<M>,
    llvm_func: &Function,
    builder: &mut FunctionBuilder,
    inst_id: InstructionId,
) {
    let inst = llvm_func.data.inst_ref(inst_id);

    match inst.operand {
        Operand::Ret(Ret { val: Some(val), ty }) => {
            let val = build_value(modules, llvm_func, builder, val, ty);
            builder.ins().return_(&[val]);
        }
        _ => {}
    }
}

fn build_value<M: Module>(
    modules: &Modules<M>,
    llvm_func: &Function,
    builder: &mut FunctionBuilder,
    val_id: ValueId,
    ty: TypeId,
) -> Value {
    match llvm_func.data.value_ref(val_id) {
        LlvmValue::Constant(ConstantData::Int(ConstantInt::Int32(i))) => {
            builder.ins().iconst(modules.into_cl_type(ty), *i as i64)
        }
        _ => todo!(),
    }
}
