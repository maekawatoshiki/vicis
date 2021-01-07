use crate::codegen::{function::Function, module::Module, target::Target};

pub fn run_on_module<T: Target>(module: &mut Module<T>) {
    for (_, func) in &mut module.functions {
        run_on_function(func);
    }
}

pub fn run_on_function<T: Target>(function: &mut Function<T>) {
    for block_id in function.layout.block_iter() {
        for inst_id in function.layout.inst_iter(block_id) {}
    }
}
