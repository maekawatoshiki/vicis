use vicis_ir::ir::{function, module};

#[test]
fn build() {
    let mut module = module::Module::default();
    let mut func = function::Function::new(
        "func",
        module.types.base().i32(),
        vec![],
        false,
        module.types.clone(),
    );
}
