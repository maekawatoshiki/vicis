use crate::{
    function::{compile_function, declare_and_define_function},
    LowerCtx,
};
use cranelift_codegen::Context;
use cranelift_module::Module;
use vicis_core::ir::module::Module as LlvmModule;

pub fn compile_module<M: Module>(clif_mod: &mut M, clif_ctx: &mut Context, llvm_mod: &LlvmModule) {
    let mut lower_ctx = LowerCtx::new(llvm_mod, clif_mod);

    let mut funcs = vec![];
    let mut protos = vec![];

    for func in llvm_mod.functions() {
        if func.1.is_prototype() {
            protos.push(func);
        } else {
            funcs.push(func);
        }
    }

    // Declare prototypes first.
    for (func_id, _) in protos {
        compile_function(&mut lower_ctx, clif_ctx, func_id);
        lower_ctx.clif_mod.clear_context(clif_ctx);
    }

    // Define functions.
    for (func_id, func) in funcs {
        compile_function(&mut lower_ctx, clif_ctx, func_id);
        declare_and_define_function(lower_ctx.clif_mod, clif_ctx, func.name().as_str());
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test1() {
        compile(
            r#"
        declare dso_local i32 @putchar(i8 signext)
        define dso_local i32 @main() {
          call i32 @putchar(i8 signext 65)
          ret i32 0
        }"#,
        );
    }

    fn compile(source: &str) {
        use cranelift::prelude::Configurable;
        use cranelift_codegen::{isa, settings};
        use cranelift_object::{ObjectBuilder, ObjectModule};
        use vicis_core::ir::module::Module;

        let mut flag_builder = settings::builder();
        flag_builder.enable("is_pic").unwrap();
        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        let isa_builder = isa::lookup_by_name("aarch64-unknown-unknown-elf").unwrap();
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
        compile_module(&mut clif_mod, &mut clif_ctx, &module);

        clif_mod.finish();
    }
}
