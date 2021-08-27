pub mod parser;

pub use parser::operand as parse_operand;
pub use parser::parse;

use crate::ir::{module::name::Name, value::ConstantInt};
use std::fmt;

#[derive(PartialEq, Clone)]
pub enum Metadata {
    String(String),
    Name(Name),
    Int(ConstantInt),
    Node(Vec<Self>),
}

// struct MetadataNode(Vec<Metadata>);

// Metadata Node

impl fmt::Debug for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn fmt_int(i: &ConstantInt, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            // TODO: Find another simpler way
            match i {
                ConstantInt::Int1(_) => write!(f, "i1 "),
                ConstantInt::Int8(_) => write!(f, "i8 "),
                ConstantInt::Int32(_) => write!(f, "i32 "),
                ConstantInt::Int64(_) => write!(f, "i64 "),
            }?;
            write!(f, "{}", i)
        }

        match self {
            Self::String(s) => write!(f, "!\"{}\"", s),
            Self::Name(n) => write!(f, "!{}", n),
            Self::Int(i) => fmt_int(i, f),
            Self::Node(list) => {
                write!(f, "!{{")?;
                for (k, m) in list.iter().enumerate() {
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
