pub mod basic_block;
pub mod data;
pub mod instruction;
pub mod layout;
pub mod slot;

use super::{call_conv::CallConvKind, isa::TargetIsa};
use crate::codegen::function::instruction::InstructionInfo;
use instruction::InstructionId;
use std::{fmt, marker::PhantomData};
use vicis_core::ir::{function::Function as IrFunction, types::Types};

pub struct Function<'a, T: TargetIsa> {
    pub ir: &'a IrFunction,
    pub data: data::Data<<T::InstInfo as InstructionInfo>::Data>,
    pub layout: layout::Layout<<T::InstInfo as InstructionInfo>::Data>,
    pub slots: slot::Slots<T>,
    pub types: Types,
    pub is_declaration: bool,
    pub call_conv: CallConvKind,
    pub _isa: PhantomData<fn() -> T>,
}

impl<T: TargetIsa> Function<'_, T> {
    pub fn remove_inst(
        &mut self,
        inst: InstructionId<<T::InstInfo as InstructionInfo>::Data>,
    ) -> Option<()> {
        self.layout.remove_inst(inst)
    }
}

impl<T: TargetIsa> fmt::Debug for Function<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_declaration {
            write!(f, "declare ")?
        } else {
            write!(f, "define ")?
        }
        write!(f, "{:?} ", self.ir.preemption_specifier)?;
        write!(f, "{} ", self.types.to_string(self.ir.result_ty))?;
        write!(f, "@{}(", self.ir.name)?;
        for (i, param) in self.ir.params.iter().enumerate() {
            write!(
                f,
                "{} %A{}{}",
                self.types.to_string(param.ty),
                i,
                if i == self.ir.params.len() - 1 {
                    ""
                } else {
                    ", "
                }
            )?;
        }
        write!(f, ") ")?;
        for attr in &self.ir.func_attrs {
            write!(f, "{:?}", attr)?;
        }

        if self.is_declaration {
            writeln!(f)?;
        } else {
            writeln!(f, "{{")?;
            for block_id in self.layout.block_iter() {
                writeln!(f, "B{:?}:", block_id.index())?;
                for inst_id in self.layout.inst_iter(block_id) {
                    let inst = self.data.inst_ref(inst_id);
                    writeln!(f, "  id{:<4}| {:?}", inst_id.index(), inst.data)?;
                }
            }
            writeln!(f, "}}")?;
        }

        Ok(())
    }
}
