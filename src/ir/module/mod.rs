pub mod attributes;
pub mod name;
pub mod parser;
pub mod preemption_specifier;

pub use parser::parse;

use super::{function::Function, types::Types};
use id_arena::Arena;
use std::fmt;

#[derive(Debug, Clone)]
struct Target {
    triple: String,
    datalayout: String,
}

pub struct Module {
    name: String,
    source_filename: String,
    target: Target,
    functions: Arena<Function>,
    types: Types,
    // TODO: Metadata
}
impl Module {
    pub fn new() -> Self {
        Self {
            name: "".to_string(),
            source_filename: "".to_string(),
            target: Target::new(),
            functions: Arena::new(),
            types: Types::new(),
        }
    }
}

impl Target {
    pub fn new() -> Self {
        Self {
            triple: "".to_string(),
            datalayout: "".to_string(),
        }
    }
}

impl fmt::Debug for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "source_filename = \"{}\"", self.source_filename)?;
        writeln!(f, "target datalayout = \"{}\"", self.target.datalayout)?;
        writeln!(f, "target triple = \"{}\"", self.target.triple)?;
        for (_, func) in &self.functions {
            writeln!(f, "{:?}", func)?;
        }
        Ok(())
    }
}
