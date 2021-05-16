use super::{function::Function, isa::TargetIsa};
use crate::ir::{
    module::{
        attributes::Attribute, global_variable::GlobalVariable, name::Name, Target as ModuleTarget,
    },
    types::Types,
};
use id_arena::Arena;
use rustc_hash::FxHashMap;
use std::fmt;

pub struct Module<T: TargetIsa> {
    pub name: String,
    pub source_filename: String,
    pub target: ModuleTarget, // TODO
    pub functions: Arena<Function<T>>,
    pub attributes: FxHashMap<u32, Vec<Attribute>>,
    pub global_variables: FxHashMap<Name, GlobalVariable>,
    pub types: Types,
    // TODO: Metadata
    pub isa: T,
}

impl<T: TargetIsa> fmt::Debug for Module<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "source_filename = \"{}\"", self.source_filename)?;
        writeln!(f, "target datalayout = \"{}\"", self.target.datalayout())?;
        writeln!(f, "target triple = \"{}\"", self.target.triple())?;
        writeln!(f)?;
        for gv in self.global_variables.values() {
            writeln!(f, "{}", gv.to_string(&self.types))?;
        }
        writeln!(f)?;
        for (_, func) in &self.functions {
            writeln!(f, "{:?}", func)?;
        }
        for (id, attrs) in &self.attributes {
            write!(f, "attributes #{} = {{ ", id)?;
            for attr in attrs {
                write!(f, "{:?} ", attr)?;
            }
            writeln!(f, "}}")?
        }
        Ok(())
    }
}
