use super::into_cl_type;
use cranelift::{
    frontend::{FunctionBuilder, FunctionBuilderContext},
    prelude::{InstBuilder, Value},
};
use cranelift_codegen::Context;
use cranelift_jit::JITModule;
use vicis_ir::ir::{
    function::{
        instruction::{InstructionId, Operand, Ret},
        Function,
    },
    module::Module,
    types::TypeId,
    value::ValueId,
    value::{ConstantData, ConstantInt, Value as LlvmValue},
};

pub fn compile_function_body(
    builder_ctx: &mut FunctionBuilderContext,
    cl_mod: &mut JITModule,
    cl_ctx: &mut Context,
    llvm_mod: &Module,
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
            compile_instruction(cl_mod, llvm_mod, llvm_func, &mut builder, inst_id);
        }
    }

    builder.finalize();
}

pub fn compile_instruction(
    cl_mod: &JITModule,
    _llvm_mod: &Module,
    llvm_func: &Function,
    builder: &mut FunctionBuilder,
    inst_id: InstructionId,
) {
    let inst = llvm_func.data.inst_ref(inst_id);

    match inst.operand {
        Operand::Ret(Ret { val: Some(val), ty }) => {
            let val = build_value(cl_mod, llvm_func, builder, val, ty);
            builder.ins().return_(&[val]);
        }
        _ => {}
    }
}

fn build_value(
    cl_mod: &JITModule,
    llvm_func: &Function,
    builder: &mut FunctionBuilder,
    val_id: ValueId,
    ty: TypeId,
) -> Value {
    match llvm_func.data.value_ref(val_id) {
        LlvmValue::Constant(ConstantData::Int(ConstantInt::Int32(i))) => builder
            .ins()
            .iconst(into_cl_type(&llvm_func.types, cl_mod, ty), *i as i64),
        _ => todo!(),
    }
}
