extern crate structopt;
extern crate vicis_core;
// extern crate vicis_interpreter;

use cranelift::prelude::Configurable;
use cranelift_codegen::{isa, settings};
use cranelift_module::Module;
use cranelift_object::{ObjectBuilder, ObjectModule};
use std::fs;
use std::io::Write;
use structopt::StructOpt;
use vicis_codegen_cranelift::function::{compile_function, declare_and_define_function};
use vicis_core::ir::module;

#[derive(Debug, StructOpt)]
#[structopt(name = "i")]
pub struct Opt {
    pub ir_file: String,

    #[structopt(long = "load")]
    pub libs: Vec<String>,
}

fn main() {
    let opt = Opt::from_args();
    let ir = fs::read_to_string(opt.ir_file).expect("failed to load *.ll file");
    let module = module::parse_assembly(ir.as_str()).expect("failed to parse LLVM Assembly");
    // let main = module
    //     .find_function_by_name("main")
    //     .expect("failed to lookup 'main'");
    // let ctx = interpreter::Context::new(&module)
    //     .with_libs(opt.libs)
    //     .expect("failed to load library");
    // let ret = interpreter::run_function(&ctx, main, vec![]);
    // process::exit(ret.expect("unknown error").sext_to_i64().unwrap_or(0) as i32)

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

    let llvm_func_id = module.find_function_by_name("main").unwrap();
    compile_function(&mut clif_mod, &mut clif_ctx, &module, llvm_func_id);
    let _id = declare_and_define_function(&mut clif_mod, &mut clif_ctx, "main");

    let product = clif_mod.finish();
    let obj = product.emit().unwrap();
    std::fs::File::create("output.o")
        .unwrap()
        .write(&obj)
        .unwrap();
}
