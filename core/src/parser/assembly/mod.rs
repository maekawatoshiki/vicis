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
pub mod util;
pub mod value;
pub mod visibility;

use nom::error::VerboseError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error<'a> {
    #[error("Parsing error occurred at {0}")]
    Located(&'a str, &'static str),
    #[error("Nom error: {0}")]
    Nom(nom::Err<VerboseError<&'a str>>),
}

impl<'a> From<nom::Err<VerboseError<&'a str>>> for Error<'a> {
    fn from(err: nom::Err<VerboseError<&'a str>>) -> Self {
        Error::Nom(err)
    }
}
