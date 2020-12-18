pub mod parser;

pub use parser::parse;

#[derive(Debug, Clone, Copy)]
pub enum PreemptionSpecifier {
    DsoPreemptable,
    DsoLocal,
}
