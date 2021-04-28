pub mod parser;

pub use parser::parse;

use std::fmt;

#[derive(Copy, Clone)]
pub enum UnnamedAddr {
    Local,
    Global,
}

impl fmt::Debug for UnnamedAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Local => write!(f, "local_unnamed_addr"),
            Self::Global => write!(f, "unnamed_addr"),
        }
    }
}
