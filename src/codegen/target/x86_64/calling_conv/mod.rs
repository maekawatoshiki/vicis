use crate::codegen::{
    calling_conv::CallingConv,
    register::Reg,
    target::x86_64::register::{RegClass, GR32, GR64},
};

#[derive(Copy, Clone)]
pub struct SystemV;

impl CallingConv<RegClass> for SystemV {
    fn gpr_list_for_rc(rc: &RegClass) -> Vec<Reg> {
        match rc {
            RegClass::GR32 => vec![GR32::EAX, GR32::ECX, GR32::EDX]
                .into_iter()
                .map(|r| r.into())
                .collect(),
            RegClass::GR64 => vec![GR64::RAX, GR64::RCX, GR64::RDX]
                .into_iter()
                .map(|r| r.into())
                .collect(),
        }
    }
}
