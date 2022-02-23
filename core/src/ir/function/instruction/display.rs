use super::{Instruction, InstructionId};
use crate::ir::{
    function::{
        data::Data,
        instruction::{Alloca, Operand},
    },
    module::name::Name,
    types::Types,
};
use std::fmt;

pub struct DisplayInstruction<'a> {
    pub inst: &'a Instruction,
    pub data: &'a Data,
    pub types: &'a Types,
    pub name_fn: Option<Box<dyn Fn(InstructionId) -> Option<Name> + 'a>>, // value name resolver
}

impl<'a> DisplayInstruction<'a> {
    pub fn set_name_fn(mut self, name_fn: Box<dyn Fn(InstructionId) -> Option<Name> + 'a>) -> Self {
        self.name_fn = Some(name_fn);
        self
    }
}

impl fmt::Display for DisplayInstruction<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let dest = self
            .name_fn
            .as_ref()
            .and_then(|f| f(self.inst.id.unwrap()))
            .unwrap_or_else(|| {
                self.inst
                    .dest
                    .to_owned()
                    .unwrap_or(Name::Number(self.inst.id.unwrap().index()))
            });

        match &self.inst.operand {
            Operand::Alloca(Alloca {
                tys,
                num_elements,
                align,
            }) => {
                write!(
                    f,
                    "%{dest:?} = alloca {}, {} {}{}",
                    self.types.to_string(tys[0]),
                    self.types.to_string(tys[1]),
                    num_elements.to_string(&self.types),
                    if *align > 0 {
                        format!(", align {}", align)
                    } else {
                        "".to_string()
                    }
                )
            }
            _ => todo!(),
        }
    }
}
