pub mod pattern;

use super::{
    function::{data::Data, layout::Layout, Function as MachFunction},
    module::Module as MachModule,
    target::Target,
};
use crate::ir::{function::Function as IrFunction, module::Module as IrModule};
use id_arena::Arena;
use rustc_hash::FxHashMap;

pub fn convert_module<T: Target>(module: IrModule) -> MachModule<T> {
    let mut functions = Arena::new();

    for (_, function) in module.functions {
        functions.alloc(convert_function::<T>(function));
    }

    MachModule {
        name: module.name,
        source_filename: module.source_filename,
        target: module.target,
        functions,
        attributes: module.attributes,
        global_variables: module.global_variables,
        types: module.types,
    }
}

pub fn convert_function<T: Target>(function: IrFunction) -> MachFunction<T> {
    let mut data: Data<T::InstData> = Data::new();
    let mut layout: Layout<T::InstData> = Layout::new();
    let mut block_map = FxHashMap::default();

    // Create machine basic blocks
    for block_id in function.layout.block_iter() {
        let new_block_id = data.create_block();
        layout.append_block(new_block_id);
        block_map.insert(block_id, new_block_id);
    }

    for block_id in function.layout.block_iter() {
        let mut mach_insts = vec![];
        for inst_id in function.layout.inst_iter(block_id).rev() {
            let pats = T::select_patterns();
            for pat in pats {
                if let Some(is) = pat(&function.data, &function.data.inst_ref(inst_id)) {
                    mach_insts.extend(is.into_iter())
                }
            }
        }
        for mach_inst in mach_insts {
            let mach_inst = data.create_inst(mach_inst);
            layout.append_inst(mach_inst, block_map[&block_id])
        }
    }

    MachFunction {
        name: function.name,
        is_var_arg: function.is_var_arg,
        result_ty: function.result_ty,
        params: function.params,
        preemption_specifier: function.preemption_specifier,
        attributes: function.attributes,
        data,
        layout,
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
