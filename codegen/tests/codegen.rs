use std::fs;
use vicis_codegen::{isa::x86_64::X86_64, lower::compile_module};
use vicis_core::ir::module::Module;

macro_rules! test {
    ($testname:ident, $name:expr) => {
        #[test]
        fn $testname() {
            let parent = "./tests/codegen/";
            let input = format!("{}{}.ll", parent, $name);
            let input_body = &fs::read_to_string(input).unwrap();
            let module = Module::try_from(input_body.as_str()).unwrap();
            let isa = X86_64::default();
            let mach_module = compile_module(&isa, &module).unwrap();
            insta::assert_display_snapshot!(mach_module.display_asm());
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
test!(test_hello, "hello");
test!(test_addr, "addr");
test!(test_i8, "i8");
