use vicis_core::ir::{function::builder::Builder, module::Module};

#[test]
fn build() {
    let mut module = Module::default();
    let int = module.types.i32();

    let func_id = module.create_function("func", int, vec![], false);
    let func = &mut module.functions_mut()[func_id];

    let mut builder = Builder::new(func);
    let entry = builder.create_block();
    builder.switch_to_block(entry);
    let forty_two = builder.value(42i32);
    builder.inst().ret(forty_two);

    insta::assert_debug_snapshot!(module);
}
