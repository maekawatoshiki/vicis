extern crate cranelift;
extern crate cranelift_jit;
extern crate cranelift_module;
extern crate vicis_ir;

use std::mem::transmute;

use cranelift::{
    codegen::binemit::{NullStackMapSink, NullTrapSink},
    frontend::{FunctionBuilder, FunctionBuilderContext},
    prelude::{AbiParam, InstBuilder},
};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module};
use vicis_ir::ir::{function::Function, module::Module as IrModule};

pub fn compile_module(_module: &IrModule) -> i32 {
    // for (_id, func) in module.functions() {
    //     compile_function(func);
    // }

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

    code_fn()
}

pub fn compile_function(_func: &Function) {}

#[test]
fn test() {
    let module = IrModule::new();
    assert_eq!(compile_module(&module), 42);
}
