use crate::ir::{
    module::{
        linkage::Linkage, name::Name, preemption_specifier::PreemptionSpecifier,
        unnamed_addr::UnnamedAddr, visibility::Visibility,
    },
    types::{Type, Types},
    value::ConstantValue,
};

#[derive(Clone)]
pub struct GlobalVariable {
    pub name: Name,
    pub linkage: Option<Linkage>,
    pub preemption_specifier: Option<PreemptionSpecifier>,
    pub visibility: Option<Visibility>,
    pub unnamed_addr: Option<UnnamedAddr>,
    pub is_constant: bool,
    pub ty: Type,
    pub init: Option<ConstantValue>,
    pub align: u32,
}

impl GlobalVariable {
    pub fn to_string(&self, types: &Types) -> String {
        format!(
            "@{} = {}{}{}{}{}{} {}{}",
            self.name,
            self.linkage
                .map_or("".to_string(), |linkage| format!("{:?} ", linkage)),
            self.preemption_specifier
                .map_or("".to_string(), |p| format!("{:?} ", p)),
            self.visibility
                .map_or("".to_string(), |v| format!("{:?} ", v)),
            self.unnamed_addr
                .map_or("".to_string(), |u| format!("{:?} ", u)),
            if self.is_constant {
                "constant "
            } else {
                "global "
            },
            types.to_string(self.ty),
            self.init.as_ref().map_or("".to_string(), |init| {
                if matches!(init, ConstantValue::AggregateZero(_)) {
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
