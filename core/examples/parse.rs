use std::fs::read_to_string;

use vicis_core::ir::module;

fn main() {
    let filename = std::env::args().nth(1).expect("expect *.ll file");
    let source = read_to_string(filename).expect("failed to load file");
    let m = module::parse_assembly(source.as_str()).expect("failed to parse file");
    println!("#### Parsed result ####\n{:?}", m);
}
