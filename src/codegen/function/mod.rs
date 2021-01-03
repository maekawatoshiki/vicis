use crate::ir::{
    function::{Parameter, UnresolvedAttributeId},
    module::{attributes::Attribute, name::Name, preemption_specifier::PreemptionSpecifier},
    types::{TypeId, Types},
};
use either::Either;

pub struct Function {
    pub(super) name: String,
    pub(super) is_var_arg: bool,
    pub(super) result_ty: TypeId,
    pub(super) params: Vec<Parameter>,
    pub(super) preemption_specifier: PreemptionSpecifier,
    pub(super) attributes: Vec<Either<Attribute, UnresolvedAttributeId>>,
    // pub data: Data,
    // pub layout: Layout,
    pub types: Types,
    is_prototype: bool,
}
