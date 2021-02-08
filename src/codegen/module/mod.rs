use super::{function::Function, isa::TargetIsa};
use crate::ir::{
    module::{
        attributes::Attribute, global_variable::GlobalVariable, name::Name, Target as ModuleTarget,
    },
    types::Types,
};
use id_arena::Arena;
use rustc_hash::FxHashMap;

pub struct Module<T: TargetIsa> {
    pub name: String,
    pub source_filename: String,
    pub target: ModuleTarget, // TODO
    pub functions: Arena<Function<T>>,
    pub attributes: FxHashMap<u32, Vec<Attribute>>,
    pub global_variables: FxHashMap<Name, GlobalVariable>,
    pub types: Types,
    // TODO: Metadata
    pub arch: T, // TODO: Should be named 'target'
}
