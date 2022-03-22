use super::{function::Function, isa::TargetIsa};
use id_arena::Arena;
use std::fmt;
use vicis_core::ir::{module::Module as IrModule, types::Types};

pub struct Module<'a, T: TargetIsa> {
    pub ir: &'a IrModule,
    pub functions: Arena<Function<'a, T>>,
    pub types: Types,
    pub isa: &'a T,
}

impl<T: TargetIsa> fmt::Debug for Module<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "source_filename = \"{}\"", self.ir.source_filename())?;
        writeln!(
            f,
            "target datalayout = \"{}\"",
            self.ir.target().datalayout()
        )?;
        writeln!(f, "target triple = \"{}\"", self.ir.target().triple())?;
        writeln!(f)?;
        for gv in self.ir.global_variables().values() {
            writeln!(f, "{}", gv.to_string(&self.types))?;
        }
        writeln!(f)?;
        for (_, func) in &self.functions {
            writeln!(f, "{:?}", func)?;
        }
        for (id, attrs) in self.ir.attributes() {
            write!(f, "attributes #{} = {{ ", id)?;
            for attr in attrs {
                write!(f, "{:?} ", attr)?;
            }
            writeln!(f, "}}")?
        }
        Ok(())
    }
}
