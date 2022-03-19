use super::{function::Function, isa::TargetIsa};
use id_arena::Arena;
use rustc_hash::FxHashMap;
use std::{fmt, marker::PhantomData};
use vicis_core::ir::{
    module::{
        attributes::Attribute, global_variable::GlobalVariable, name::Name, Module as IrModule,
    },
    types::Types,
};

pub struct Module<'a, T: TargetIsa> {
    pub module: &'a IrModule,
    pub functions: Arena<Function<'a, T>>,
    pub attributes: FxHashMap<u32, Vec<Attribute>>,
    pub global_variables: FxHashMap<Name, GlobalVariable>,
    pub types: Types,
    pub _isa: PhantomData<fn() -> T>,
}

impl<T: TargetIsa> fmt::Debug for Module<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "source_filename = \"{}\"", self.module.source_filename())?;
        writeln!(
            f,
            "target datalayout = \"{}\"",
            self.module.target().datalayout()
        )?;
        writeln!(f, "target triple = \"{}\"", self.module.target().triple())?;
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
