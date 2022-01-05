extern crate structopt;
extern crate vicis_core;

use cranelift::prelude::Configurable;
use cranelift_codegen::{isa, settings};
use cranelift_module::Module;
use cranelift_object::{ObjectBuilder, ObjectModule};
use std::fs;
use std::io::Write;
use structopt::StructOpt;
use vicis_codegen_cranelift::module::compile_module;
use vicis_core::ir::module;

#[derive(Debug, StructOpt)]
#[structopt(name = "i")]
pub struct Opt {
    pub ir_file: String,

    #[structopt(short = "o")]
    pub output_file: Option<String>,
}

fn main() {
    let opt = Opt::from_args();
    let ir = fs::read_to_string(opt.ir_file).expect("failed to load *.ll file");
    let module = module::parse_assembly(ir.as_str()).expect("failed to parse LLVM Assembly");

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

    compile_module(&mut clif_mod, &mut clif_ctx, &module);

    let product = clif_mod.finish();
    let obj = product.emit().unwrap();
    std::fs::File::create(opt.output_file.unwrap_or("a.o".to_owned()))
        .unwrap()
        .write(&obj)
        .unwrap();
}
