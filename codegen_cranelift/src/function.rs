use crate::{instruction, LowerCtx};
use cranelift::{
    codegen::binemit::{NullStackMapSink, NullTrapSink},
    frontend::{FunctionBuilder, FunctionBuilderContext},
    prelude::AbiParam,
};
use cranelift_codegen::Context;
use cranelift_module::{FuncId, Linkage, Module};
use rustc_hash::FxHashMap;
use vicis_core::ir::{
    function::{Function, FunctionId},
    module::Module as LlvmModule,
};

/// Compiles a llvm function to a cranelift function.
pub fn compile_function<M: Module>(
    clif_mod: &mut M,
    clif_ctx: &mut Context,
    llvm_mod: &LlvmModule,
    llvm_func_id: FunctionId,
) {
    let llvm_func = &llvm_mod.functions()[llvm_func_id];
    let mut builder_ctx = FunctionBuilderContext::new();
    let mut lower_ctx = LowerCtx::new(llvm_mod, clif_mod);

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

    compile_body(&mut lower_ctx, &mut builder_ctx, clif_ctx, llvm_func);
}

/// An utility function to declare and define a cranelift function.
pub fn declare_and_define_function<M: Module>(clif_mod: &mut M, clif_ctx: &mut Context) -> FuncId {
    let id = clif_mod
        .declare_function("func", Linkage::Export, &clif_ctx.func.signature)
        .unwrap();
    clif_mod
        .define_function(id, clif_ctx, &mut NullTrapSink {}, &mut NullStackMapSink {})
        .unwrap();
    clif_mod.clear_context(clif_ctx);
    id
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn compile_func() {
        use cranelift::prelude::Configurable;
        use cranelift_codegen::isa::CallConv;
        use cranelift_codegen::{isa, settings};
        use cranelift_object::{ObjectBuilder, ObjectModule};
        use vicis_core::ir::module;

        let source = r#"
define dso_local i32 @main() {
  ret i32 42
}"#;

        let mut flag_builder = settings::builder();
        flag_builder.enable("is_pic").unwrap();
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        let isa_builder = isa::lookup_by_name("x86_64-unknown-unknown-elf").unwrap();
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        let isa_builder = isa::lookup_by_name("aarch64-apple-darwin").unwrap();
        let isa = isa_builder.finish(settings::Flags::new(flag_builder));

        let builder = ObjectBuilder::new(
            isa,
            "".to_owned(), // TODO: This will be embedded in the object file.
            cranelift_module::default_libcall_names(),
        )
        .unwrap();
        let mut clif_mod = ObjectModule::new(builder);
        let mut clif_ctx = clif_mod.make_context();

        let module = module::parse_assembly(source).unwrap();
        let llvm_func_id = module.find_function_by_name("main").unwrap();
        compile_function(&mut clif_mod, &mut clif_ctx, &module, llvm_func_id);

        // TODO: FIXME: Depending on OS and ISA, the calling convention may vary.
        // This makes it difficult to do testing using insta because the text representation
        // of a cranelift function contains the name of calling convention.
        // So we have to reset the calling convention here.
        clif_ctx.func.signature.call_conv = CallConv::Fast;
        insta::assert_display_snapshot!(clif_ctx.func.display());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn compile_ret_42() {
        use cranelift_jit::{JITBuilder, JITModule};
        use std::mem::transmute;
        use vicis_core::ir::module;

        let source = r#"
define dso_local i32 @main() {
  ret i32 42
}"#;

        let builder = JITBuilder::new(cranelift_module::default_libcall_names());
        let mut clif_mod = JITModule::new(builder);
        let mut clif_ctx = clif_mod.make_context();

        let module = module::parse_assembly(source).unwrap();
        let func_id = module.find_function_by_name("main").unwrap();
        compile_function(&mut clif_mod, &mut clif_ctx, &module, func_id);
        let func_id = declare_and_define_function(&mut clif_mod, &mut clif_ctx);
        clif_mod.finalize_definitions();

        let code = clif_mod.get_finalized_function(func_id);
        let code_fn = unsafe { transmute::<_, fn() -> i32>(code) };
        assert_eq!(code_fn(), 42);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn compile_add() {
        use cranelift_jit::{JITBuilder, JITModule};
        use std::mem::transmute;
        use vicis_core::ir::module;

        let source = r#"
define dso_local i32 @main(i32 %arg.0) {
  %result = add nsw i32 %arg.0, 1
  ret i32 %result
}"#;

        let builder = JITBuilder::new(cranelift_module::default_libcall_names());
        let mut clif_mod = JITModule::new(builder);
        let mut clif_ctx = clif_mod.make_context();

        let module = module::parse_assembly(source).unwrap();
        let func_id = module.find_function_by_name("main").unwrap();
        compile_function(&mut clif_mod, &mut clif_ctx, &module, func_id);
        let id = declare_and_define_function(&mut clif_mod, &mut clif_ctx);
        clif_mod.finalize_definitions();

        let code = clif_mod.get_finalized_function(id);
        let code_fn = unsafe { transmute::<_, fn(i32) -> i32>(code) };
        assert_eq!(code_fn(41), 42);
    }
}
