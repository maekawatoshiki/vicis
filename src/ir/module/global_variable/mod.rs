mod parser;

pub use parser::parse;

use crate::ir::{
    module::{linkage::Linkage, name::Name},
    types::{TypeId, Types},
    value::ConstantData,
};

pub struct GlobalVariable {
    pub name: Name,
    pub linkage: Option<Linkage>,
    pub is_local_unnamed_addr: bool, // unnamed_addr or local_unnamed_addr
    pub is_constant: bool,
    pub ty: TypeId,
    pub init: Option<ConstantData>,
    pub align: u32,
}

impl GlobalVariable {
    pub fn to_string(&self, types: &Types) -> String {
        format!(
            "@{:?} = {}{}{}{} {}, align {}",
            self.name,
            self.linkage
                .map_or("".to_string(), |linkage| format!("{:?} ", linkage)),
            if self.is_local_unnamed_addr {
                "local_unnamed_addr "
            } else {
                "unnamed_addr "
            },
            if self.is_constant {
                "constant "
            } else {
                "global "
            },
            types.to_string(self.ty),
            self.init
                .as_ref()
                .map_or("".to_string(), |init| { init.to_string(types) }),
            self.align
        )
    }
}
