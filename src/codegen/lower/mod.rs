pub mod pattern;

use super::{
    function::{data::Data, layout::Layout, slot::Slots, Function as MachFunction},
    instruction::InstructionData,
    module::Module as MachModule,
    register::VRegs,
    target::Target,
};
use crate::ir::{
    function::{Data as IrData, Function as IrFunction},
    instruction::Instruction as IrInstruction,
    module::Module as IrModule,
};
use id_arena::Arena;
use pattern::{Lower, LoweringContext};
use rustc_hash::FxHashMap;

pub struct Context<'a, InstData: InstructionData> {
    pub ir_data: &'a IrData,
    pub inst: &'a IrInstruction,
    pub mach_data: &'a mut Data<InstData>,
}

pub fn convert_module<T: Target>(target: T, module: IrModule) -> MachModule<T> {
    let mut functions = Arena::new();

    for (_, function) in module.functions {
        functions.alloc(convert_function(target, function));
    }

    let mut mach_module = MachModule {
        name: module.name,
        source_filename: module.source_filename,
        target: module.target,
        functions,
        attributes: module.attributes,
        global_variables: module.global_variables,
        types: module.types,
        arch: target,
    };

    for pass in target.module_pass() {
        pass(&mut mach_module)
    }

    mach_module
}

pub fn convert_function<T: Target>(target: T, function: IrFunction) -> MachFunction<T> {
    let mut slots: Slots<T> = Slots::new(target);
    let mut data: Data<T::InstData> = Data::new();
    let mut layout: Layout<T::InstData> = Layout::new();
    let mut block_map = FxHashMap::default();

    // Create machine basic blocks
    for block_id in function.layout.block_iter() {
        let new_block_id = data.create_block();
        layout.append_block(new_block_id);
        block_map.insert(block_id, new_block_id);
    }

    // Insert preds and succs
    for block_id in function.layout.block_iter() {
        let new_block_id = block_map[&block_id];
        let block = &function.data.basic_blocks[block_id];
        let new_block = data.basic_blocks.get_mut(new_block_id).unwrap();
        for pred in &block.preds {
            new_block.preds.insert(block_map[pred]);
        }
        for succ in &block.succs {
            new_block.succs.insert(block_map[succ]);
        }
    }

    let mut inst_id_to_slot_id = FxHashMap::default();
    let mut inst_id_to_vreg = FxHashMap::default();
    let mut vregs = VRegs::new();

    for block_id in function.layout.block_iter() {
        let mut inst_seq = vec![];
        for inst_id in function.layout.inst_iter(block_id) {
            let inst = function.data.inst_ref(inst_id);

            // `inst` is used once in current block
            // `inst` is used many times in current block
            // inst.users.

            target.lower().lower(
                &mut LoweringContext {
                    ir_data: &function.data,
                    mach_data: &mut data,
                    slots: &mut slots,
                    inst_id_to_slot_id: &mut inst_id_to_slot_id,
                    inst_seq: &mut inst_seq,
                    types: &function.types,
                    vregs: &mut vregs,
                    inst_id_to_vreg: &mut inst_id_to_vreg,
                },
                inst,
            );
        }
        for mach_inst in inst_seq {
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
        slots,
        vregs,
        types: function.types,
        is_prototype: function.is_prototype,
        target,
    }
}
