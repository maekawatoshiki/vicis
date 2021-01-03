use super::{function::Function, target::Target};
use crate::ir::{
    module::{
        attributes::Attribute, global_variable::GlobalVariable, name::Name, Target as ModuleTarget,
    },
    types::Types,
};
use id_arena::Arena;
use rustc_hash::FxHashMap;

pub struct Module<T: Target> {
    pub(super) name: String,
    pub(super) source_filename: String,
    pub(super) target: ModuleTarget,
    pub(super) functions: Arena<Function<T>>,
    pub(super) attributes: FxHashMap<u32, Vec<Attribute>>,
    pub(super) global_variables: FxHashMap<Name, GlobalVariable>,
    pub types: Types,
    // TODO: Metadata
}
