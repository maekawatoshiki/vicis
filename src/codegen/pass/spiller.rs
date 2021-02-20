use super::liveness::Liveness;
use crate::{
    codegen::{
        function::{instruction::InstructionInfo, Function},
        isa::TargetIsa,
        register::VReg,
    },
    ir::types::Type,
};

pub struct Spiller<'a, T: TargetIsa> {
    function: &'a mut Function<T>,
    liveness: &'a mut Liveness<T>,
}

impl<'a, T: TargetIsa> Spiller<'a, T> {
    pub fn new(function: &'a mut Function<T>, liveness: &'a mut Liveness<T>) -> Self {
        Self { function, liveness }
    }

    pub fn spill(&mut self, vreg: VReg) {
        self.insert_spill(vreg);
        // self.insert_reload(vreg);
    }

    fn insert_spill(&mut self, vreg: VReg) {
        // let slot_id = ctx
        //     .slots
        //     .add_slot(tys[0], X86_64::type_size(ctx.types, tys[0]));
        let defs = &self.liveness.vreg_to_defs[&vreg];

        if defs.len() == 0 {
            return;
        }

        let ty = self.function.vregs.type_for(vreg);
        assert!(&*self.function.types.get(ty) == &Type::Int(32));
        let slot = self
            .function
            .slots
            .add_slot(ty, T::type_size(&self.function.types, ty));

        // Most cases
        if defs.len() == 1 {
            let def_id = *defs.iter().next().unwrap();
            let def_block = self.function.data.instructions[def_id].parent;
            let inst = T::InstInfo::store_vreg_to_slot(self.function, vreg, slot, def_block);
            let inst = self.function.data.create_inst(inst);
            self.function
                .layout
                .insert_inst_after(def_id, inst, def_block);
        }

        // Two addr instruction
        if defs.len() == 2 {}

        panic!("invalid")
    }
}
