extern crate structopt;
extern crate vicis_codegen;
extern crate vicis_core;

use std::fs::{self, File};
use std::io::Write;
use structopt::StructOpt;
use vicis_codegen::lower::compile_module;
use vicis_core::ir::function::Function;
use vicis_core::ir::module::Module;
use vicis_core::pass::transform::mem2reg::Mem2RegPass;
use vicis_core::pass::PassManager;

#[derive(Debug, StructOpt)]
#[structopt(name = "compile")]
pub struct Opt {
    pub ir_file: String,

    #[structopt(short = "o", help = "Output file name")]
    pub out_file: Option<String>,

    #[structopt(
        short = "p",
        long = "pass",
        help = "Specify a comma-separated list of passes to run"
    )]
    pub pass: Option<String>,
}

fn main() {
    env_logger::init();
    color_backtrace::install();

    let opt = Opt::from_args();
    let ir = fs::read_to_string(&opt.ir_file).expect("failed to load *.ll file");
    let mut module = Module::try_from(ir.as_str()).expect("failed to parse LLVM Assembly");

    let mut pm = PassManager::new();
    if let Some(pass) = opt.pass {
        set_passes(&mut pm, pass);
        pm.run_on_module(&mut module);
    }

    #[cfg(target_arch = "x86_64")]
    let isa = {
        use vicis_codegen::isa::x86_64::X86_64;
        X86_64::default()
    };
    #[cfg(target_arch = "aarch64")]
    let isa = {
        use vicis_codegen::isa::aarch64::Aarch64;
        Aarch64::default()
    };
    let module = compile_module(&isa, &module).expect("failed to compile");
    File::create(
        opt.out_file
            .unwrap_or_else(|| opt.ir_file.trim_end_matches(".ll").to_owned() + ".s"),
    )
    .expect("failed to create output file")
    .write(format!("{}", module.display_asm()).as_bytes())
    .unwrap();
}

fn set_passes(pm: &mut PassManager<Function>, pass: String) {
    for name in pass.split(',') {
        match name {
            "mem2reg" => pm.add_transform(Mem2RegPass),
            "" => continue,
            _ => panic!("Unknown pass: {}", name),
        }
    }
}
