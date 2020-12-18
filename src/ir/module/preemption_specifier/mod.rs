pub mod parser;

pub use parser::parse;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PreemptionSpecifier {
    DsoPreemptable,
    DsoLocal,
}
