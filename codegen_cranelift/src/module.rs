use crate::function::compile_function;
use cranelift_codegen::Context;
use cranelift_module::{Linkage, Module};
use vicis_core::ir::module::Module as LlvmModule;

pub fn compile_module<M: Module>(clif_mod: &mut M, clif_ctx: &mut Context, llvm_mod: &LlvmModule) {
    for (func_id, func) in llvm_mod.functions() {
        if func.is_prototype() {
            // dbg!(func);
            let sig = clif_mod.make_signature();
            clif_mod
                .declare_function(func.name().as_str(), Linkage::Import, &sig)
                .expect("?");
            continue;
        }

        compile_function(clif_mod, clif_ctx, llvm_mod, func_id)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test1() {
        // compile(
        //     r#"
        // declare dso_local i32 @putchar(i8 signext)
        // define dso_local i32 @main() {
        //   call i32 @putchar(i8 signext 65)
        //   ret i32 0
        // }"#,
        // );
    }

    #[allow(dead_code)]
    fn compile(source: &str) {
        use cranelift::prelude::Configurable;
        use cranelift_codegen::{isa, settings};
        use cranelift_object::{ObjectBuilder, ObjectModule};
        use vicis_core::ir::module;

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
        compile_module(&mut clif_mod, &mut clif_ctx, &module);
    }
}
