pub mod data;
pub mod layout;

use super::target::Target;
use crate::ir::{
    function::{Parameter, UnresolvedAttributeId},
    module::{attributes::Attribute, preemption_specifier::PreemptionSpecifier},
    types::{TypeId, Types},
};
use either::Either;

pub struct Function<T: Target> {
    pub name: String,
    pub is_var_arg: bool,
    pub result_ty: TypeId,
    pub params: Vec<Parameter>,
    pub preemption_specifier: PreemptionSpecifier,
    pub attributes: Vec<Either<Attribute, UnresolvedAttributeId>>,
    pub data: data::Data<T::InstData>,
    pub layout: layout::Layout<T::InstData>,
    pub types: Types,
    pub is_prototype: bool,
}
