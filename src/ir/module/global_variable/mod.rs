mod parser;

pub use parser::parse;

use crate::ir::{
    module::{linkage::Linkage, name::Name},
    types::TypeId,
};

pub struct GlobalVariable {
    pub name: Name,
    pub linkage: Option<Linkage>,
    pub is_local_unnamed_addr: bool, // unnamed_addr or local_unnamed_addr
    pub is_constant: bool,
    pub ty: TypeId,
}
