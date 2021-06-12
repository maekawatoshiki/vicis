extern crate rand;
extern crate structopt;
extern crate vicis;

use rand::Rng;
use std::{fs, io::Write, process};
use structopt::StructOpt;
use vicis::{
    codegen::{isa::x86_64::X86_64, lower::compile_module},
    ir::module,
};

#[derive(Debug, StructOpt)]
#[structopt(name = "i")]
pub struct Opt {
    pub ir_file: String,

    #[structopt(long = "load")]
    pub libs: Vec<String>,
}

fn main() {
    let opt = Opt::from_args();
    let ir = fs::read_to_string(opt.ir_file.as_str()).expect("failed to load *.ll file");
    let module = module::parse_assembly(ir.as_str()).expect("failed to parse LLVM Assembly");
    let module = compile_module(X86_64, module).expect("failed to compile module");
    let asm_file_name = unique_file_name("s");
    let mut output =
        fs::File::create(asm_file_name.as_str()).expect("failed to create output *.s file");
    write!(output, "{}", module).unwrap();
    output.flush().unwrap();
    let exe_file_name = unique_file_name("out");
    assert!(process::Command::new("clang")
        .args(&[asm_file_name.as_str(), "-o", exe_file_name.as_str()])
        .status()
        .unwrap()
        .success());
    let ret = process::Command::new(exe_file_name.as_str())
        .status()
        .unwrap();
    process::exit(ret.code().expect("failed to run"));
}

fn unique_file_name(extension: &str) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";
    const LEN: usize = 16;
    let mut rng = rand::thread_rng();
    let name: String = (0..LEN)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    format!("/tmp/{}.{}", name, extension)
}
