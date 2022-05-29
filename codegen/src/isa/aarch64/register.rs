use vicis_core::ir::types::{Type, Types};

use crate::{
    call_conv::CallConvKind,
    register::{Reg, RegUnit, RegisterClass, RegisterInfo},
};

pub struct RegInfo;

pub enum RegClass {
    GPR,
}

impl RegisterClass for RegClass {
    fn for_type(_types: &Types, _ty: Type) -> Self {
        todo!()
    }

    fn gpr_list(&self) -> Vec<Reg> {
        todo!()
    }

    fn csr_list(&self) -> Vec<Reg> {
        todo!()
    }

    fn apply_for(&self, _ru: RegUnit) -> Reg {
        todo!()
    }
}

impl RegisterInfo for RegInfo {
    fn arg_reg_list(_cc: &CallConvKind) -> &'static [RegUnit] {
        todo!()
    }

    fn to_reg_unit(_r: Reg) -> RegUnit {
        todo!()
    }

    fn is_csr(_: RegUnit) -> bool {
        todo!()
    }
}
