pub mod attributes;
pub mod global_variable;
pub mod linkage;
pub mod metadata;
pub mod name;
pub mod preemption_specifier;
pub mod unnamed_addr;
pub mod visibility;

use super::{
    function::{Function, FunctionId, Parameter},
    types::{Type, Types},
};
use attributes::Attribute;
use global_variable::GlobalVariable;
use id_arena::{Arena, Id};
use metadata::Metadata;
use name::Name;
use rustc_hash::FxHashMap;
use std::fmt;

#[derive(Debug, Clone, Default)]
pub struct Target {
    pub triple: String,
    pub datalayout: String,
}

pub struct Module {
    pub(crate) name: String,
    pub(crate) source_filename: String,
    pub(crate) target: Target,
    pub(crate) functions: Arena<Function>,
    pub(crate) attributes: FxHashMap<u32, Vec<Attribute>>,
    pub(crate) global_variables: FxHashMap<Name, GlobalVariable>,
    pub types: Types,
    pub metas: FxHashMap<Name, Metadata>,
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

    pub fn add_function(&mut self, f: Function) -> Id<Function> {
        self.functions.alloc(f)
    }

    pub fn create_function<T: AsRef<str>>(
        &mut self,
        name: T,
        result_ty: Type,
        params: Vec<Parameter>,
        is_var_arg: bool,
    ) -> Id<Function> {
        self.functions.alloc(Function::new(
            name,
            result_ty,
            params,
            is_var_arg,
            self.types.clone(),
        ))
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

impl<'a> TryFrom<&'a str> for Module {
    type Error = crate::parser::assembly::Error<'a>;

    /// Parses an LLVM Assembly string.
    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        use crate::parser::assembly::module::parse;
        parse(s)
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
