extern crate vicis_core;
extern crate wasm_bindgen;

pub mod generic_value;
pub mod interpreter;

use vicis_core::ir::module::Module;
use wasm_bindgen::prelude::*;

// #[wasm_bindgen]
// extern "C" {
//     pub fn alert(s: &str);
// }

#[wasm_bindgen]
pub fn greet(ir: &str) -> String {
    let module = Module::try_from(ir).expect("failed to parse LLVM Assembly");
    let main = module
        .find_function_by_name("main")
        .expect("failed to lookup 'main'");
    let ctx = interpreter::ContextBuilder::new(&module)
        .build()
        .expect("failed to create interpreter context");
    let ret = interpreter::run_function(&ctx, main, vec![]).unwrap();
    format!("{:?}", ret)
}
