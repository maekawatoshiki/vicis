extern crate cranelift;
extern crate cranelift_codegen;
extern crate cranelift_jit;
extern crate cranelift_module;
extern crate vicis_ir;

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
use vicis_ir::ir::{
    function::FunctionId,
    module::Module as LlvmModule,
    types::{Type as LlvmType, TypeId as LlvmTypeId, Types},
};

// pub fn compile_module(_module: &IrModule) {
//     // for (_id, func) in module.functions() {
//     //     compile_function(func);
//     // }
// }

pub fn compile_function(llvm_mod: &LlvmModule, llvm_func_id: FunctionId) -> *const u8 {
    let llvm_func = &llvm_mod.functions()[llvm_func_id];

    let builder = JITBuilder::new(cranelift_module::default_libcall_names());
    let mut builder_ctx = FunctionBuilderContext::new();
    let mut cl_mod = JITModule::new(builder);
    let mut cl_ctx = cl_mod.make_context();

    // let mut ctx = Context {
    //     llvm_mod,
    //     builder_ctx,
    //     cl_mod,
    //     cl_ctx,
    // };

    for param in &llvm_func.params {
        let cl_ty = into_cl_type(&llvm_mod.types, &cl_mod, param.ty);
        cl_ctx.func.signature.params.push(AbiParam::new(cl_ty));
    }
    // cl_ctx.func.signature.params.push(AbiParam::new(int));
    let cl_result_ty = into_cl_type(&llvm_mod.types, &cl_mod, llvm_func.result_ty);
    cl_ctx
        .func
        .signature
        .returns
        .push(AbiParam::new(cl_result_ty));

    instruction::compile_function_body(
        &mut builder_ctx,
        &mut cl_mod,
        &mut cl_ctx,
        llvm_mod,
        llvm_func,
    );

    dbg!(&cl_ctx.func);

    let id = cl_mod
        .declare_function("func", Linkage::Export, &cl_ctx.func.signature)
        .unwrap();
    cl_mod
        .define_function(
            id,
            &mut cl_ctx,
            &mut NullTrapSink {},
            &mut NullStackMapSink {},
        )
        .unwrap();
    cl_mod.clear_context(&mut cl_ctx);
    cl_mod.finalize_definitions();
    let code = cl_mod.get_finalized_function(id);
    code
}

fn into_cl_type(types: &Types, cl_mod: &JITModule, ty: LlvmTypeId) -> Type {
    match *types.get(ty) {
        LlvmType::Int(32) => types::I32,
        LlvmType::Int(64) => types::I64,
        LlvmType::Pointer(_) => cl_mod.target_config().pointer_type(),
        _ => todo!(),
    }
}

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

#[test]
fn test2() {
    use vicis_ir::ir::module;

    let source = r#"
define dso_local i32 @main() {
  ret i32 42
}"#;

    let module = module::parse_assembly(source).unwrap();
    let func_id = module.find_function_by_name("main").unwrap();
    compile_function(&module, func_id);
}
