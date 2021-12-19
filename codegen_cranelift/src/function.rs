use crate::{instruction, LowerCtx};
use cranelift::{
    codegen::binemit::{NullStackMapSink, NullTrapSink},
    frontend::{FunctionBuilder, FunctionBuilderContext},
    prelude::AbiParam,
};
use cranelift_codegen::Context;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module};
use rustc_hash::FxHashMap;
use vicis_core::ir::{
    function::{Function, FunctionId},
    module::Module as LlvmModule,
};

pub fn compile_function(llvm_mod: &LlvmModule, llvm_func_id: FunctionId) -> *const u8 {
    let llvm_func = &llvm_mod.functions()[llvm_func_id];

    let builder = JITBuilder::new(cranelift_module::default_libcall_names());
    let mut builder_ctx = FunctionBuilderContext::new();
    let mut clif_mod = JITModule::new(builder);
    let mut clif_ctx = clif_mod.make_context();

    let mut lower_ctx = LowerCtx::new(llvm_mod, &mut clif_mod);

    for param in &llvm_func.params {
        let clif_ty = lower_ctx.into_clif_ty(param.ty);
        clif_ctx.func.signature.params.push(AbiParam::new(clif_ty));
    }
    let clif_result_ty = lower_ctx.into_clif_ty(llvm_func.result_ty);
    clif_ctx
        .func
        .signature
        .returns
        .push(AbiParam::new(clif_result_ty));

    compile_body(&mut lower_ctx, &mut builder_ctx, &mut clif_ctx, llvm_func);

    dbg!(&clif_ctx.func);

    let id = clif_mod
        .declare_function("func", Linkage::Export, &clif_ctx.func.signature)
        .unwrap();
    clif_mod
        .define_function(
            id,
            &mut clif_ctx,
            &mut NullTrapSink {},
            &mut NullStackMapSink {},
        )
        .unwrap();
    clif_mod.clear_context(&mut clif_ctx);
    clif_mod.finalize_definitions();
    let code = clif_mod.get_finalized_function(id);
    code
}

fn compile_body<M: Module>(
    lower_ctx: &mut LowerCtx<'_, M>,
    builder_ctx: &mut FunctionBuilderContext,
    cl_ctx: &mut Context,
    llvm_func: &Function,
) {
    let mut builder = FunctionBuilder::new(&mut cl_ctx.func, builder_ctx);
    let mut compiler = instruction::InstCompiler {
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

#[cfg(target_os = "linux")]
#[test]
fn test() {
    use std::mem::transmute;

    use cranelift::{
        codegen::binemit::{NullStackMapSink, NullTrapSink},
        frontend::{FunctionBuilder, FunctionBuilderContext},
        prelude::{AbiParam, InstBuilder},
    };

    let builder = JITBuilder::new(cranelift_module::default_libcall_names());
    let mut builder_ctx = FunctionBuilderContext::new();
    let mut module = JITModule::new(builder);
    let mut ctx = module.make_context();
    // let data_ctx = DataContext::new();

    let int = module.target_config().pointer_type();
    ctx.func.signature.params.push(AbiParam::new(int));
    ctx.func.signature.returns.push(AbiParam::new(int));

    {
        let mut builder = FunctionBuilder::new(&mut ctx.func, &mut builder_ctx);
        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);
        let forty_two = builder.ins().iconst(int, 42);
        builder.ins().return_(&[forty_two]);
        builder.finalize();
    }

    println!("{}", ctx.func);

    let id = module
        .declare_function("func", Linkage::Export, &ctx.func.signature)
        .unwrap();
    module
        .define_function(id, &mut ctx, &mut NullTrapSink {}, &mut NullStackMapSink {})
        .unwrap();
    module.clear_context(&mut ctx);
    module.finalize_definitions();
    let code = module.get_finalized_function(id);
    let code_fn = unsafe { transmute::<_, fn() -> i32>(code) };

    assert_eq!(code_fn(), 42);
}

#[cfg(target_os = "linux")]
#[test]
fn compile_ret_42() {
    use std::mem::transmute;
    use vicis_core::ir::module;

    let source = r#"
define dso_local i32 @main() {
  ret i32 42
}"#;

    let module = module::parse_assembly(source).unwrap();
    let func_id = module.find_function_by_name("main").unwrap();
    let code = compile_function(&module, func_id);
    let code_fn = unsafe { transmute::<_, fn() -> i32>(code) };
    assert_eq!(code_fn(), 42);
}

#[cfg(target_os = "linux")]
#[test]
fn compile_add() {
    use std::mem::transmute;
    use vicis_core::ir::module;

    let source = r#"
define dso_local i32 @main(i32 %arg.0) {
  %result = add nsw i32 %arg.0, 1
  ret i32 %result
}"#;

    let module = module::parse_assembly(source).unwrap();
    let func_id = module.find_function_by_name("main").unwrap();
    let code = compile_function(&module, func_id);
    let code_fn = unsafe { transmute::<_, fn(i32) -> i32>(code) };
    assert_eq!(code_fn(41), 42);
}
