pub mod data;
pub mod layout;

use super::{
    basic_block::{BasicBlock, BasicBlockId},
    instruction::{Instruction, InstructionId},
};
use crate::ir::{
    function::{Parameter, UnresolvedAttributeId},
    module::{attributes::Attribute, preemption_specifier::PreemptionSpecifier},
    types::{TypeId, Types},
};
use either::Either;
use id_arena::Arena;
use rustc_hash::FxHashMap;

pub struct Function {
    pub(super) name: String,
    pub(super) is_var_arg: bool,
    pub(super) result_ty: TypeId,
    pub(super) params: Vec<Parameter>,
    pub(super) preemption_specifier: PreemptionSpecifier,
    pub(super) attributes: Vec<Either<Attribute, UnresolvedAttributeId>>,
    pub data: data::Data,
    pub layout: layout::Layout,
    pub types: Types,
    pub is_prototype: bool,
}
