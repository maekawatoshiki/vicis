use super::{
    function::{data::Data, layout::Layout, Function as MachFunction},
    module::Module as MachModule,
};
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
    let mut data = Data::new();
    let mut layout = Layout::new();

    for block_id in function.layout.block_iter() {
        // for inst_id in function.layout.inst_iter(block_id) {}
    }

    MachFunction {
        name: function.name,
        is_var_arg: function.is_var_arg,
        result_ty: function.result_ty,
        params: function.params,
        preemption_specifier: function.preemption_specifier,
        attributes: function.attributes,
        data: Data::new(),
        layout: Layout::new(),
        types: function.types,
        is_prototype: function.is_prototype,
    }
}

// IrInstruction -> MachInstruction (vreg)? Either?
// enum InstructionData {
//     MOVri32 {
//         dst: Either<GR32, Vreg32>,
//         src: Imm32,
//     },
// }
