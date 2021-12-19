extern crate cranelift;
extern crate cranelift_codegen;
extern crate cranelift_jit;
extern crate cranelift_module;
extern crate vicis_core;

pub mod function;
mod instruction;

use cranelift::codegen::{ir::types, ir::types::Type};
use cranelift_module::Module;
use vicis_core::ir::{
    module::Module as LlvmModule,
    types::{Type as LlvmType, TypeId as LlvmTypeId},
};

pub struct LowerCtx<'a, M: Module> {
    llvm_mod: &'a LlvmModule,
    clif_mod: &'a mut M,
}

impl<'a, M: Module> LowerCtx<'a, M> {
    pub fn new(llvm_mod: &'a LlvmModule, clif_mod: &'a mut M) -> Self {
        Self { llvm_mod, clif_mod }
    }

    pub fn into_clif_ty(&self, ty: LlvmTypeId) -> Type {
        match *self.llvm_mod.types.get(ty) {
            LlvmType::Int(32) => types::I32,
            LlvmType::Int(64) => types::I64,
            LlvmType::Pointer(_) => self.clif_mod.target_config().pointer_type(),
            _ => todo!(),
        }
    }
}
