use crate::ir::types::{self, Type, Typed};

use super::{Instruction, Operand};

impl Typed for Instruction {
    fn ty(&self) -> Type {
        self.operand.ty()
    }
}

impl Typed for Operand {
    fn ty(&self) -> Type {
        match self {
            Self::Ret(_) => types::VOID,
            _ => types::VOID,
        }
    }
}
