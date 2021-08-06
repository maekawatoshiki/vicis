mod parser;

pub use parser::{parse, parse_global_type_and_const};

use crate::ir::{
    module::{linkage::Linkage, name::Name, unnamed_addr::UnnamedAddr},
    types::{TypeId, Types},
    value::ConstantData,
};

#[derive(Clone)]
pub struct GlobalVariable {
    pub name: Name,
    pub linkage: Option<Linkage>,
    pub unnamed_addr: Option<UnnamedAddr>,
    pub is_constant: bool,
    pub ty: TypeId,
    pub init: Option<ConstantData>,
    pub align: u32,
}

impl GlobalVariable {
    pub fn to_string(&self, types: &Types) -> String {
        format!(
            "@{} = {}{}{}{} {}{}",
            self.name,
            self.linkage
                .map_or("".to_string(), |linkage| format!("{:?} ", linkage)),
            self.unnamed_addr
                .map_or("".to_string(), |u| format!("{:?} ", u)),
            if self.is_constant {
                "constant "
            } else {
                "global "
            },
            types.to_string(self.ty),
            self.init.as_ref().map_or("".to_string(), |init| {
                if matches!(init, ConstantData::AggregateZero) {
                    "zeroinitializer".to_string()
                } else {
                    init.to_string(types)
                }
            }),
            if self.align == 0 {
                "".to_string()
            } else {
                format!(", align {}", self.align)
            }
        )
    }
}
