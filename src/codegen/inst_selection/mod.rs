use super::{function::Function as MachFunction, module::Module as MachModule};
use crate::ir::{function::Function as IrFunction, module::Module as IrModule};
use id_arena::Arena;

pub fn convert_module(module: IrModule) -> MachModule {
    MachModule {
        name: module.name,
        source_filename: module.source_filename,
        target: module.target,
        functions: Arena::new(),
        attributes: module.attributes,
        global_variables: module.global_variables,
        types: module.types,
    }
}

pub fn convert_function(function: IrFunction) -> MachFunction {
    todo!()
}

// IrInstruction -> MachInstruction (vreg)? Either?
// enum InstructionData {
//     MOVri32 {
//         dst: Either<GR32, Vreg32>,
//         src: Imm32,
//     },
// }
