pub mod attributes;
pub mod global_variable;
pub mod linkage;
pub mod name;
pub mod parser;
pub mod preemption_specifier;
pub mod unnamed_addr;
pub mod visibility;

pub use parser::parse as parse_assembly;

use super::{
    function::{Function, FunctionId},
    types::Types,
};
use crate::ir::value::ConstantInt;
use attributes::Attribute;
use global_variable::GlobalVariable;
use id_arena::Arena;
use name::Name;
use rustc_hash::FxHashMap;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Target {
    triple: String,
    datalayout: String,
}

#[derive(PartialEq, Clone)]
pub enum Meta {
    String(String),
    Name(Name),
    Int(ConstantInt),
    Metas(Vec<Meta>),
}

pub struct Module {
    pub(crate) name: String,
    pub(crate) source_filename: String,
    pub(crate) target: Target,
    pub(crate) functions: Arena<Function>,
    pub(crate) attributes: FxHashMap<u32, Vec<Attribute>>,
    pub(crate) global_variables: FxHashMap<Name, GlobalVariable>,
    pub types: Types,
    pub metas: FxHashMap<Name, Meta>,
}

impl Default for Module {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            source_filename: "".to_string(),
            target: Target::new(),
            functions: Arena::new(),
            attributes: FxHashMap::default(),
            global_variables: FxHashMap::default(),
            types: Types::new(),
            metas: FxHashMap::default(),
        }
    }
}

impl Module {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn source_filename(&self) -> &String {
        &self.source_filename
    }

    pub fn target(&self) -> &Target {
        &self.target
    }

    pub fn functions(&self) -> &Arena<Function> {
        &self.functions
    }

    pub fn functions_mut(&mut self) -> &mut Arena<Function> {
        &mut self.functions
    }

    pub fn attributes(&self) -> &FxHashMap<u32, Vec<Attribute>> {
        &self.attributes
    }

    pub fn global_variables(&self) -> &FxHashMap<Name, GlobalVariable> {
        &self.global_variables
    }

    pub fn find_function_by_name<T: AsRef<str>>(&self, name: T) -> Option<FunctionId> {
        for (id, func) in &self.functions {
            if func.name() == name.as_ref() {
                return Some(id);
            }
        }
        None
    }
}

impl Default for Target {
    fn default() -> Self {
        Self {
            triple: "".to_string(),
            datalayout: "".to_string(),
        }
    }
}

impl Target {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn triple(&self) -> &str {
        self.triple.as_str()
    }

    pub fn datalayout(&self) -> &str {
        self.datalayout.as_str()
    }
}

impl fmt::Debug for Meta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn fmt_metaint(s: &ConstantInt, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match s {
                ConstantInt::Int1(i) => write!(f, "i1 {}", i),
                ConstantInt::Int8(i) => write!(f, "i8 {}", i),
                ConstantInt::Int32(i) => write!(f, "i32 {}", i),
                ConstantInt::Int64(i) => write!(f, "i64 {}", i),
            }
        }
        match self {
            Meta::String(s) => write!(f, "!\"{}\"", s),
            Meta::Name(n) => write!(f, "!{}", n),
            Meta::Int(n) => fmt_metaint(n, f),
            Meta::Metas(ms) => {
                write!(f, r"!{{")?;
                for (k, m) in ms.iter().enumerate() {
                    if k == 0 {
                        write!(f, "{:?}", m)?;
                    } else {
                        write!(f, ", {:?}", m)?;
                    }
                }
                write!(f, "}}")
            }
        }
    }
}

impl fmt::Debug for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "source_filename = \"{}\"", self.source_filename)?;
        writeln!(f, "target datalayout = \"{}\"", self.target.datalayout)?;
        writeln!(f, "target triple = \"{}\"", self.target.triple)?;
        writeln!(f)?;
        write!(f, "{:?}", self.types)?;
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
        for (n, meta) in &self.metas {
            writeln!(f, "!{} = {:?}", n, meta)?;
        }
        Ok(())
    }
}
