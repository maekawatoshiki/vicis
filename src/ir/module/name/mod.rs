pub mod parser;

pub use parser::parse;

use std::fmt;

#[derive(Clone, Eq, PartialEq)]
pub enum Name {
    Name(String),
    Number(usize),
}

impl fmt::Debug for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Name(name) => write!(f, "{}", name),
            Self::Number(num) => write!(f, "{}", num),
        }
    }
}
