pub mod attributes;
pub mod function;
pub mod global_variable;
pub mod instruction;
pub mod linkage;
pub mod metadata;
pub mod module;
pub mod name;
pub mod param_attrs;
pub mod preemption_specifier;
pub mod types;
pub mod unnamed_addr;
pub mod value;
pub mod visibility;

use nom::error::VerboseError;

#[derive(Debug)]
pub enum Error<'a> {
    Located(&'a str, &'static str),
    Nom(nom::Err<VerboseError<&'a str>>),
}

impl<'a> From<nom::Err<VerboseError<&'a str>>> for Error<'a> {
    fn from(err: nom::Err<VerboseError<&'a str>>) -> Self {
        Error::Nom(err)
    }
}

impl std::error::Error for Error<'_> {}

impl std::fmt::Display for Error<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}
