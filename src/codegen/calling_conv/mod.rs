use crate::codegen::register::{Reg, RegisterClass};

pub trait CallingConv<RC: RegisterClass>: Copy {
    fn gpr_list_for_rc(rc: &RC) -> Vec<Reg>;
}
