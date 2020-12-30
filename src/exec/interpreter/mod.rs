use crate::ir::{function::FunctionId, module::Module};

pub struct Interpreter<'a> {
    module: &'a Module,
}

impl<'a> Interpreter<'a> {
    pub fn new(module: &'a Module) -> Self {
        Self { module }
    }

    pub fn run_function(&mut self, _func_id: FunctionId) {}
}
