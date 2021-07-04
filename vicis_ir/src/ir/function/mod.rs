pub mod basic_block;
pub mod data;
pub mod instruction;
pub mod layout;
pub mod param_attrs;
pub mod parser;
pub mod print;

pub use parser::parse;

use super::{
    module::{
        attributes::Attribute, linkage::Linkage, name::Name,
        preemption_specifier::PreemptionSpecifier, unnamed_addr::UnnamedAddr,
        visibility::Visibility,
    },
    types::{TypeId, Types},
    value::ConstantData,
};
use crate::traits::basic_block::{BasicBlockData, BasicBlockLayout};
use basic_block::BasicBlock;
use id_arena::Id;
use instruction::InstructionId;
use param_attrs::ParameterAttribute;
use std::fmt;

pub type FunctionId = Id<Function>;

pub type PersonalityFunc = (TypeId, ConstantData);

pub struct Function {
    pub name: String,
    pub is_var_arg: bool,
    pub result_ty: TypeId,
    pub params: Vec<Parameter>,
    pub linkage: Linkage,
    pub preemption_specifier: PreemptionSpecifier,
    pub visibility: Visibility,
    pub unnamed_addr: Option<UnnamedAddr>,
    pub func_attrs: Vec<Attribute>,
    pub ret_attrs: Vec<param_attrs::ParameterAttribute>,
    pub personality: Option<PersonalityFunc>,
    pub data: data::Data,
    pub layout: layout::Layout,
    pub types: Types,
    pub is_prototype: bool,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: Name,
    pub ty: TypeId,
    pub attrs: Vec<ParameterAttribute>,
}

impl Function {
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn params(&self) -> &[Parameter] {
        &self.params
    }

    pub fn is_var_arg(&self) -> bool {
        self.is_var_arg
    }

    pub fn remove_inst(&mut self, inst: InstructionId) -> Option<()> {
        self.data.remove_uses(inst);
        self.layout.remove_inst(inst)
    }
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        print::FunctionAsmPrinter::new(f).print(self)
    }
}

impl Parameter {
    pub fn to_string(&self, types: &Types) -> String {
        format!("{} %{:?}", types.to_string(self.ty), self.name)
    }
}

impl BasicBlockData<BasicBlock> for Function {
    fn get(&self, id: Id<BasicBlock>) -> &BasicBlock {
        &self.data.basic_blocks[id]
    }
}

impl BasicBlockLayout<BasicBlock> for Function {
    fn order(&self) -> Box<dyn Iterator<Item = Id<BasicBlock>> + '_> {
        Box::new(self.layout.block_iter())
    }
}
