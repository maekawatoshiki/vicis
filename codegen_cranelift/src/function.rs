use crate::{instruction, LowerCtx};
use cranelift::{
    codegen::binemit::{NullStackMapSink, NullTrapSink},
    frontend::{FunctionBuilder, FunctionBuilderContext},
    prelude::AbiParam,
};
use cranelift_codegen::Context;
use cranelift_module::{FuncId, Linkage, Module};
use rustc_hash::FxHashMap;
use vicis_core::ir::function::{Function, FunctionId};

/// Compiles a llvm function to a cranelift function.
pub fn compile_function<M: Module>(
    lower_ctx: &mut LowerCtx<'_, M>,
    clif_ctx: &mut Context,
    llvm_func_id: FunctionId,
) {
    let llvm_func = &lower_ctx.llvm_mod.functions()[llvm_func_id];
    let mut builder_ctx = FunctionBuilderContext::new();

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

    if llvm_func.is_prototype() {
        lower_ctx
            .clif_mod
            .declare_function(
                llvm_func.name().as_str(),
                Linkage::Import,
                &clif_ctx.func.signature,
            )
            .unwrap();
        return;
    }

    compile_body(lower_ctx, &mut builder_ctx, clif_ctx, llvm_func);

    #[cfg(debug_assertions)]
    dbg!(&clif_ctx.func);
}

/// An utility function to declare and define a cranelift function.
pub fn declare_and_define_function<M: Module>(
    clif_mod: &mut M,
    clif_ctx: &mut Context,
    name: impl AsRef<str>,
) -> FuncId {
    let id = clif_mod
        .declare_function(name.as_ref(), Linkage::Export, &clif_ctx.func.signature)
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
        stack_slots: FxHashMap::default(),
    };

    // Create all basic blocks.
    for block_id in llvm_func.layout.block_iter() {
        compiler.create_block_for(block_id);
    }

    for (i, block_id) in llvm_func.layout.block_iter().enumerate() {
        let block = compiler.blocks[&block_id];
        if i == 0 {
            compiler
                .builder
                .append_block_params_for_function_params(block);
        }
        compiler.builder.switch_to_block(block);
        for inst_id in llvm_func.layout.inst_iter(block_id) {
            compiler.compile(inst_id);
        }
    }

    // Seal all basic blocks.
    for block_id in llvm_func.layout.block_iter() {
        let block = compiler.blocks[&block_id];
        compiler.builder.seal_block(block);
    }

    compiler.builder.finalize();
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn compile_func() {
        let f = compile_main(
            r#"
        define dso_local i32 @main() {
          ret i32 42
        }"#,
        );
        insta::assert_display_snapshot!(f.display());
    }

    #[test]
    fn compile_func2() {
        let f = compile_main(
            r#"
        define dso_local i32 @main(i32 returned %0) local_unnamed_addr #0 {
          ret i32 %0
        }"#,
        );
        insta::assert_display_snapshot!(f.display());
    }

    #[test]
    fn compile_func3() {
        let f = compile_main(
            r#"
        define dso_local i32 @main(i32 %0) local_unnamed_addr #0 {
          %2 = add nsw i32 %0, 1
          ret i32 %2
        }"#,
        );
        insta::assert_display_snapshot!(f.display());
    }

    #[test]
    fn compile_func4() {
        let f = compile_main(
            r#"
        define dso_local i32 @main() #0 {
          %1 = alloca i32, align 4
          store i32 123, i32* %1, align 4
          %2 = load i32, i32* %1, align 4
          ret i32 %2
        }"#,
        );
        insta::assert_display_snapshot!(f.display());
    }

    #[test]
    fn compile_func5() {
        let f = compile_main(
            r#"
        define dso_local i32 @main(i32 %0) #0 {
          %a = add nsw i32 %0, 1
          br label %l
        l:
          ret i32 %a
        }"#,
        );
        insta::assert_display_snapshot!(f.display());
    }

    #[test]
    fn compile_func6() {
        let f = compile_main(
            r#"
        define dso_local i32 @main(i32 %0) #0 {
          br i1 true, label %l, label %r
        l:
          ret i32 %0
        r:
          ret i32 0
        }"#,
        );
        insta::assert_display_snapshot!(f.display());
    }

    #[test]
    fn compile_func7() {
        let f = compile_main(
            r#"
define dso_local i32 @main() #0 {
  %1 = alloca i32, align 4
  %2 = alloca i32, align 4
  %3 = alloca i32, align 4
  store i32 0, i32* %1, align 4
  store i32 0, i32* %2, align 4
  store i32 1, i32* %3, align 4
  br label %4

4:                                                ; preds = %11, %0
  %5 = load i32, i32* %3, align 4
  %6 = icmp sle i32 %5, 10
  br i1 %6, label %7, label %14

7:                                                ; preds = %4
  %8 = load i32, i32* %3, align 4
  %9 = load i32, i32* %2, align 4
  %10 = add nsw i32 %9, %8
  store i32 %10, i32* %2, align 4
  br label %11

11:                                               ; preds = %7
  %12 = load i32, i32* %3, align 4
  %13 = add nsw i32 %12, 1
  store i32 %13, i32* %3, align 4
  br label %4

14:                                               ; preds = %4
  %15 = load i32, i32* %2, align 4
  ret i32 %15
}
        "#,
        );
        insta::assert_display_snapshot!(f.display());
    }

    fn compile_main(source: &str) -> cranelift_codegen::ir::Function {
        use cranelift::prelude::Configurable;
        use cranelift_codegen::isa::CallConv;
        use cranelift_codegen::{isa, settings};
        use cranelift_object::{ObjectBuilder, ObjectModule};
        use vicis_core::ir::module::Module;

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

        let module = Module::try_from(source).unwrap();
        let llvm_func_id = module.find_function_by_name("main").unwrap();
        compile_function(
            &mut LowerCtx::new(&module, &mut clif_mod),
            &mut clif_ctx,
            llvm_func_id,
        );

        // TODO: FIXME: Depending on OS and ISA, the calling convention may vary.
        // This makes it difficult to do testing using insta because the text representation
        // of a cranelift function contains the name of calling convention.
        // So we have to reset the calling convention here.
        clif_ctx.func.signature.call_conv = CallConv::Fast;
        clif_ctx.func
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn compile_ret_42() {
        use cranelift_jit::{JITBuilder, JITModule};
        use std::mem::transmute;
        use vicis_core::ir::module::Module;

        let source = r#"
define dso_local i32 @main() {
  ret i32 42
}"#;

        let builder = JITBuilder::new(cranelift_module::default_libcall_names());
        let mut clif_mod = JITModule::new(builder);
        let mut clif_ctx = clif_mod.make_context();

        let module = Module::try_from(source).unwrap();
        let func_id = module.find_function_by_name("main").unwrap();
        compile_function(
            &mut LowerCtx::new(&module, &mut clif_mod),
            &mut clif_ctx,
            func_id,
        );
        let func_id = declare_and_define_function(&mut clif_mod, &mut clif_ctx, "func");
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
        use vicis_core::ir::module::Module;

        let source = r#"
define dso_local i32 @main(i32 %arg.0) {
  %result = add nsw i32 %arg.0, 1
  ret i32 %result
}"#;

        let builder = JITBuilder::new(cranelift_module::default_libcall_names());
        let mut clif_mod = JITModule::new(builder);
        let mut clif_ctx = clif_mod.make_context();

        let module = Module::try_from(source).unwrap();
        let func_id = module.find_function_by_name("main").unwrap();
        compile_function(
            &mut LowerCtx::new(&module, &mut clif_mod),
            &mut clif_ctx,
            func_id,
        );
        let id = declare_and_define_function(&mut clif_mod, &mut clif_ctx, "func");
        clif_mod.finalize_definitions();

        let code = clif_mod.get_finalized_function(id);
        let code_fn = unsafe { transmute::<_, fn(i32) -> i32>(code) };
        assert_eq!(code_fn(41), 42);
    }
}
