use vicis_ir::ir::{function, module};

#[test]
fn build() {
    let module = module::Module::default();
    let mut func = function::Function::new(
        "func",
        module.types.base().i32(),
        vec![],
        false,
        module.types.clone(),
    );
    let mut builder = function::builder::Builder::new(&mut func);
    let entry = builder.create_block();
    builder.append_block(entry);
}
