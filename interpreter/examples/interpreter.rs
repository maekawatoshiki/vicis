extern crate structopt;
extern crate vicis_core;
extern crate vicis_interpreter;

use std::{fs, process};
use structopt::StructOpt;
use vicis_core::ir::module::Module;
use vicis_interpreter::interpreter;

#[derive(Debug, StructOpt)]
#[structopt(name = "i")]
pub struct Opt {
    pub ir_file: String,

    #[structopt(long = "load")]
    pub libs: Vec<String>,
}

fn main() {
    env_logger::init();
    color_backtrace::install();
    let opt = Opt::from_args();
    let ir = fs::read_to_string(opt.ir_file).expect("failed to load *.ll file");
    let module = Module::try_from(ir.as_str()).expect("failed to parse LLVM Assembly");
    let main = module
        .find_function_by_name("main")
        .expect("failed to lookup 'main'");
    let ctx = interpreter::ContextBuilder::new(&module)
        .with_libs(opt.libs)
        .build()
        .expect("failed to create interpreter context");
    let ret = interpreter::run_function(&ctx, main, vec![]);
    process::exit(ret.expect("unknown error").sext_to_i64().unwrap_or(0) as i32)
}
