use crate::codegen::{function::Function, module::Module, register::VReg, target::Target};
use id_arena::{Arena, Id};
use rustc_hash::FxHashMap;

pub struct Liveness {
    map: FxHashMap<VReg, LiveRange>,
    pp_arena: Arena<ProgramPoint>,
}

pub struct LiveRange {
    start: ProgramPointId,
    end: ProgramPointId,
}

pub type ProgramPointId = Id<ProgramPoint>;

pub struct ProgramPoint {}

// pub fn run_on_module<T: Target>(module: &mut Module<T>) {
//     for (_, func) in &mut module.functions {
//         run_on_function(func);
//     }
// }

pub fn run_on_function<T: Target>(function: &mut Function<T>) -> Liveness {
    todo!()
}
