use super::function::Function;
use crate::ir::{
    module::{attributes::Attribute, global_variable::GlobalVariable, name::Name, Target},
    types::Types,
};
use id_arena::Arena;
use rustc_hash::FxHashMap;

pub struct Module {
    pub(super) name: String,
    pub(super) source_filename: String,
    pub(super) target: Target,
    pub(super) functions: Arena<Function>,
    pub(super) attributes: FxHashMap<u32, Vec<Attribute>>,
    pub(super) global_variables: FxHashMap<Name, GlobalVariable>,
    pub types: Types,
    // TODO: Metadata
}
