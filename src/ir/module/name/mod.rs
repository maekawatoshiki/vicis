pub mod parser;

pub use parser::parse;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Name {
    Name(String),
    Number(usize),
}
