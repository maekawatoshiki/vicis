use std::fs;
use vicis_codegen::codegen::{
    isa::x86_64::X86_64, lower::compile_module, module::Module as MachModule,
};
use vicis_core::ir::module::Module;

macro_rules! test {
    ($testname:ident, $name:expr) => {
        #[test]
        fn $testname() {
            let mach_module = compile($name);
            insta::assert_display_snapshot!(mach_module);
        }
    };
}

test!(test_add, "ary1");
test!(test_ary2, "ary2");
test!(test_ary3, "ary3");
test!(test_ary4, "ary4");
test!(test_ary5, "ary5");
test!(test_br, "br");
test!(test_call1, "call1");
test!(test_call2, "call2");
test!(test_condbr, "condbr");
test!(test_fibo, "fibo");
test!(test_load_add, "load_add");
test!(test_phi, "phi");
test!(test_phi2, "phi2");
test!(test_puts, "puts");
test!(test_sum, "sum");

#[cfg(test)]
fn compile(name: &str) -> MachModule<X86_64> {
    let parent = "./tests/codegen/";
    let input = format!("{}{}.ll", parent, name);
    let input_body = &fs::read_to_string(input).unwrap();
    let module = Module::try_from(input_body.as_str()).unwrap();
    compile_module(X86_64, &module).unwrap()
}
