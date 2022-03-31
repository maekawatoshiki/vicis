extern crate structopt;
extern crate vicis_codegen;
extern crate vicis_core;

use std::fs::{self, File};
use std::io::Write;
use structopt::StructOpt;
use vicis_codegen::{isa::x86_64::X86_64, lower::compile_module};
use vicis_core::ir::module::Module;

#[derive(Debug, StructOpt)]
#[structopt(name = "i")]
pub struct Opt {
    pub ir_file: String,

    #[structopt(short = "o")]
    pub out_file: Option<String>,
}

fn main() {
    env_logger::init();
    let opt = Opt::from_args();
    let ir = fs::read_to_string(&opt.ir_file).expect("failed to load *.ll file");
    let module = Module::try_from(ir.as_str()).expect("failed to parse LLVM Assembly");
    let isa = X86_64::default();
    let module = compile_module(&isa, &module).unwrap();
    File::create(
        opt.out_file
            .unwrap_or_else(|| opt.ir_file.trim_end_matches(".ll").to_owned() + ".s"),
    )
    .expect("failed to create output file")
    .write(format!("{}", module).as_bytes())
    .unwrap();
}
