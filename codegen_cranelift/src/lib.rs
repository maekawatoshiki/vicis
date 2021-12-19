extern crate cranelift;
extern crate cranelift_codegen;
extern crate cranelift_jit;
extern crate cranelift_module;
extern crate vicis_core;

mod instruction;

use cranelift::{
    codegen::{
        binemit::{NullStackMapSink, NullTrapSink},
        ir::types,
        ir::types::Type,
    },
    frontend::FunctionBuilderContext,
    prelude::AbiParam,
};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module};
use vicis_core::ir::{
    function::FunctionId,
    module::Module as LlvmModule,
    types::{Type as LlvmType, TypeId as LlvmTypeId},
};

pub struct LowerCtx<'a, M: Module> {
    llvm_mod: &'a LlvmModule,
    clif_mod: &'a mut M,
}

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

    instruction::compile_function_body(&mut lower_ctx, &mut builder_ctx, &mut clif_ctx, llvm_func);

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

impl<'a, M: Module> LowerCtx<'a, M> {
    pub fn new(llvm_mod: &'a LlvmModule, clif_mod: &'a mut M) -> Self {
        Self { llvm_mod, clif_mod }
    }

    pub fn into_clif_ty(&self, ty: LlvmTypeId) -> Type {
        match *self.llvm_mod.types.get(ty) {
            LlvmType::Int(32) => types::I32,
            LlvmType::Int(64) => types::I64,
            LlvmType::Pointer(_) => self.clif_mod.target_config().pointer_type(),
            _ => todo!(),
        }
    }
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
