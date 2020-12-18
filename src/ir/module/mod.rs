pub mod parser;

pub use parser::parse;

use super::types::Types;

#[derive(Debug, Clone)]
struct Target {
    triple: String,
    datalayout: String,
}

#[derive(Debug, Clone)]
pub struct Module {
    name: String,
    source_filename: String,
    target: Target,
    types: Types,
    // TODO: Metadata
}

impl Module {
    pub fn new() -> Self {
        Self {
            name: "".to_string(),
            source_filename: "".to_string(),
            target: Target::new(),
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
