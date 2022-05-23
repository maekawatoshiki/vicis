use std::fs;
use vicis_codegen::{isa::x86_64::X86_64, lower::compile_module};
use vicis_core::ir::module::Module;

macro_rules! test {
    ($isa:ident, $testname:ident, $name:expr) => {
        #[test]
        fn $testname() {
            let parent = "./tests/codegen/";
            let input = format!("{}{}.ll", parent, $name);
            let input_body = &fs::read_to_string(input).unwrap();
            let module = Module::try_from(input_body.as_str()).unwrap();
            let isa = $isa::default();
            let mach_module = compile_module(&isa, &module).unwrap();
            insta::assert_display_snapshot!(mach_module.display_asm());
        }
    };
}

#[cfg(test)]
mod x86_64 {
    use super::*;

    test!(X86_64, test_add, "ary1");
    test!(X86_64, test_ary2, "ary2");
    test!(X86_64, test_ary3, "ary3");
    test!(X86_64, test_ary4, "ary4");
    test!(X86_64, test_ary5, "ary5");
    test!(X86_64, test_br, "br");
    test!(X86_64, test_call1, "call1");
    test!(X86_64, test_call2, "call2");
    test!(X86_64, test_condbr, "condbr");
    test!(X86_64, test_fibo, "fibo");
    test!(X86_64, test_load_add, "load_add");
    test!(X86_64, test_phi, "phi");
    test!(X86_64, test_phi2, "phi2");
    test!(X86_64, test_puts, "puts");
    test!(X86_64, test_sum, "sum");
    test!(X86_64, test_hello, "hello");
    test!(X86_64, test_addr, "addr");
    test!(X86_64, test_i8, "i8");
    test!(X86_64, test_i8_load_store, "i8_load_store");
    test!(X86_64, test_global, "global");
    test!(X86_64, test_spill, "spill");
}
