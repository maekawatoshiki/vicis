pub mod basic_block;
pub mod data;
pub mod instruction;
pub mod layout;
pub mod param_attrs;
pub mod parser;
pub mod print;

pub use parser::parse;

use super::{
    module::{attributes::Attribute, name::Name, preemption_specifier::PreemptionSpecifier},
    types::{TypeId, Types},
};
use crate::traits::basic_block::{BasicBlockData, BasicBlockLayout};
use basic_block::BasicBlock;
use id_arena::Id;
use instruction::InstructionId;
use std::fmt;

pub type FunctionId = Id<Function>;

pub struct Function {
    pub(crate) name: String,
    pub(crate) is_var_arg: bool,
    pub(crate) result_ty: TypeId,
    pub(crate) params: Vec<Parameter>,
    pub(crate) preemption_specifier: PreemptionSpecifier,
    pub(crate) func_attrs: Vec<Attribute>,
    pub(crate) ret_attrs: Vec<param_attrs::ParameterAttribute>,
    pub data: data::Data,
    pub layout: layout::Layout,
    pub types: Types,
    pub(crate) is_prototype: bool,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: Name,
    pub ty: TypeId,
    // pub attributes:
}

impl Function {
    pub fn name(&self) -> &String {
        &self.name
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
