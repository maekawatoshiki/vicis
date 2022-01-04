extern crate cranelift;
extern crate cranelift_codegen;
extern crate cranelift_module;
extern crate vicis_core;

pub mod function;
mod instruction;

use cranelift::codegen::{ir::types, ir::types::Type};
use cranelift_module::Module;
use vicis_core::ir::{module::Module as LlvmModule, types::Type as LlvmType};

pub struct LowerCtx<'a, M: Module> {
    llvm_mod: &'a LlvmModule,
    clif_mod: &'a mut M,
}

impl<'a, M: Module> LowerCtx<'a, M> {
    pub fn new(llvm_mod: &'a LlvmModule, clif_mod: &'a mut M) -> Self {
        Self { llvm_mod, clif_mod }
    }

    pub fn into_clif_ty(&self, ty: LlvmType) -> Type {
        if ty.is_i1() {
            return types::I8;
        }

        if ty.is_i32() {
            return types::I32;
        }

        if ty.is_i64() {
            return types::I64;
        }

        if ty.is_pointer(&self.llvm_mod.types) {
            return self.clif_mod.target_config().pointer_type();
        }

        todo!()
    }
}
