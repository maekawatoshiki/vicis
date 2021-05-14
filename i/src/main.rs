extern crate structopt;
extern crate vicis;

use std::fs;
use structopt::StructOpt;
use vicis::{exec::interpreter, ir::module};

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
    let main = module
        .find_function_by_name("main")
        .expect("failed to lookup 'main'");
    let ctx = interpreter::Context::new(&module)
        .with_libs(opt.libs)
        .expect("failed to load library");
    interpreter::run_function(&ctx, main, vec![]);
}
