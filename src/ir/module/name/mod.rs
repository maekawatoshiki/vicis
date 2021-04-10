pub mod parser;

pub use parser::parse;

use std::fmt;

#[derive(Clone, Eq, PartialEq, Hash)]
pub enum Name {
    Name(String),
    Number(usize),
}

impl Name {
    pub fn as_string(&self) -> &String {
        match self {
            Self::Name(name) => name,
            _ => panic!(),
        }
    }

    pub fn as_number(&self) -> &usize {
        match self {
            Self::Number(n) => n,
            _ => panic!(),
        }
    }

    pub fn to_string(&self) -> Option<&String> {
        match self {
            Self::Name(name) => Some(name),
            _ => None,
        }
    }
}

impl fmt::Debug for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Name(name) => write!(f, "{}", name),
            Self::Number(num) => write!(f, "{}", num),
        }
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Name(name) => write!(f, "{}", name),
            Self::Number(num) => write!(f, "{}", num),
        }
    }
}
