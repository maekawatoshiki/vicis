pub mod basic_block;
pub mod data;
pub mod instruction;
pub mod layout;
pub mod parser;

pub use parser::parse;

use super::{
    module::{attributes::Attribute, name::Name, preemption_specifier::PreemptionSpecifier},
    types::{TypeId, Types},
};
use crate::traits::basic_block::{BasicBlockData, BasicBlockLayout};
use basic_block::BasicBlock;
use either::Either;
use id_arena::Id;
use instruction::InstructionId;
use std::fmt;

/// `Attribute Group`s may be attached to `Function`s in the form of `#0`
pub type UnresolvedAttributeId = u32;

pub type FunctionId = Id<Function>;

pub struct Function {
    pub(crate) name: String,
    pub(crate) is_var_arg: bool,
    pub(crate) result_ty: TypeId,
    pub(crate) params: Vec<Parameter>,
    pub(crate) preemption_specifier: PreemptionSpecifier,
    pub(crate) attributes: Vec<Either<Attribute, UnresolvedAttributeId>>,
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
        if self.is_prototype {
            write!(f, "declare ")?
        } else {
            write!(f, "define ")?
        }
        write!(f, "{:?} ", self.preemption_specifier)?;
        write!(f, "{} ", self.types.to_string(self.result_ty))?;
        write!(f, "@{}(", self.name)?;
        for (i, param) in self.params.iter().enumerate() {
            write!(
                f,
                "{} %A{}{}",
                self.types.to_string(param.ty),
                i,
                if i == self.params.len() - 1 { "" } else { ", " }
            )?;
        }
        write!(f, ") ")?;
        for attr in &self.attributes {
            match attr {
                Either::Left(attr) => write!(f, "{:?} ", attr)?,
                Either::Right(id) => write!(f, "#{} ", id)?,
            }
        }

        if self.is_prototype {
            writeln!(f)?;
        } else {
            write!(f, "{{\n")?;
            for block_id in self.layout.block_iter() {
                writeln!(f, "B{:?}:", block_id.index())?;
                for inst_id in self.layout.inst_iter(block_id) {
                    let inst = self.data.inst_ref(inst_id);
                    writeln!(f, "    {}", inst.to_string(&self.data, &self.types))?;
                }
            }
            write!(f, "}}\n")?;
        }

        Ok(())
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
