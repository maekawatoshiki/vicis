use crate::codegen::calling_conv::CallingConv;

#[derive(Copy, Clone)]
pub struct SystemV;

impl CallingConv for SystemV {}
