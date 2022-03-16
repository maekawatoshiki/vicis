use crate::ir::{
    module::name::Name,
    types::{Typed, Types},
    value::ConstantValue,
};
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum Metadata {
    String(String),
    Name(Name),
    Const(ConstantValue),
    Node(Vec<Self>, bool /* is distinct */),
}

impl Metadata {
    pub fn display<'a>(&'a self, types: &'a Types) -> DisplayMetadata<'a> {
        DisplayMetadata { meta: self, types }
    }
}

pub struct DisplayMetadata<'a> {
    pub meta: &'a Metadata,
    pub types: &'a Types,
}

impl fmt::Display for DisplayMetadata<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.meta {
            Metadata::String(s) => write!(f, "!\"{}\"", s),
            Metadata::Name(n) => write!(f, "!{}", n),
            Metadata::Const(c) => write!(
                f,
                "{} {}",
                self.types.to_string(c.ty()),
                c.to_string(self.types)
            ),
            Metadata::Node(list, distinct) => {
                write!(f, "{}!{{", if *distinct { "distinct " } else { "" })?;
                for (k, m) in list.iter().enumerate() {
                    if k == 0 {
                        write!(f, "{}", m.display(self.types))?;
                    } else {
                        write!(f, ", {}", m.display(self.types))?;
                    }
                }
                write!(f, "}}")
            }
        }
    }
}
