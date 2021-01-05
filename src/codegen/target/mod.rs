pub mod x86_64;

use crate::codegen::{lower, module::Module};

pub trait Target: Copy {
    type InstData: ::std::fmt::Debug;
    type Lower: lower::pattern::Lower<Self>;

    fn lower(&self) -> &Self::Lower;
    fn module_pass(&self) -> Vec<fn(&mut Module<Self>)>;
}
