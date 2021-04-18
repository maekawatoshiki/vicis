pub mod basic_block;
pub mod data;
pub mod instruction;
pub mod layout;
pub mod slot;

use super::{call_conv::CallConvKind, isa::TargetIsa};
use crate::codegen::function::instruction::InstructionInfo;
use crate::ir::{
    function::Parameter,
    module::{attributes::Attribute, preemption_specifier::PreemptionSpecifier},
    types::{TypeId, Types},
};
use instruction::InstructionId;
use std::fmt;

pub struct Function<T: TargetIsa> {
    pub name: String,
    pub is_var_arg: bool,
    pub result_ty: TypeId,
    pub params: Vec<Parameter>,
    pub preemption_specifier: PreemptionSpecifier,
    pub attributes: Vec<Attribute>,
    pub data: data::Data<<T::InstInfo as InstructionInfo>::Data>,
    pub layout: layout::Layout<<T::InstInfo as InstructionInfo>::Data>,
    pub slots: slot::Slots<T>,
    pub types: Types,
    pub is_prototype: bool,
    pub call_conv: CallConvKind,
    pub isa: T,
}

impl<T: TargetIsa> Function<T> {
    pub fn remove_inst(
        &mut self,
        inst: InstructionId<<T::InstInfo as InstructionInfo>::Data>,
    ) -> Option<()> {
        self.layout.remove_inst(inst)
    }
}

impl<T: TargetIsa> fmt::Debug for Function<T> {
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
            write!(f, "{:?}", attr)?;
        }

        if self.is_prototype {
            writeln!(f)?;
        } else {
            write!(f, "{{\n")?;
            for block_id in self.layout.block_iter() {
                writeln!(f, "B{:?}:", block_id.index())?;
                for inst_id in self.layout.inst_iter(block_id) {
                    let inst = self.data.inst_ref(inst_id);
                    writeln!(f, "  id{:<4}| {:?}", inst_id.index(), inst.data)?;
                }
            }
            write!(f, "}}\n")?;
        }

        Ok(())
    }
}
