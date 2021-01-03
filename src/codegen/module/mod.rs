use super::{function::Function, target::Target};
use crate::ir::{
    module::{
        attributes::Attribute, global_variable::GlobalVariable, name::Name, Target as ModuleTarget,
    },
    types::Types,
};
use id_arena::Arena;
use rustc_hash::FxHashMap;
use std::marker::PhantomData;

pub struct Module<T: Target> {
    pub name: String,
    pub source_filename: String,
    pub target: ModuleTarget,
    pub functions: Arena<Function<T>>,
    pub attributes: FxHashMap<u32, Vec<Attribute>>,
    pub global_variables: FxHashMap<Name, GlobalVariable>,
    pub types: Types,
    // TODO: Metadata
    pub phantom: PhantomData<T>,
}
