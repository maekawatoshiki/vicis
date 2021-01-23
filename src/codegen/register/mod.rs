use crate::{
    codegen::call_conv::CallConvKind,
    ir::types::{TypeId, Types},
};
use rustc_hash::FxHashMap;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub struct Reg(pub u16, pub u16);

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub struct RegUnit(pub u16, pub u16); // Same as top-level register. TODO: This is not actually register unit

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VReg(pub u32);

pub struct VRegs {
    pub map: FxHashMap<VReg, VRegData>,
    pub cur: u32,
}

pub struct VRegData {
    pub vreg: VReg,
    pub ty: TypeId,
    // ...
}

pub trait RegisterClass {
    fn for_type(types: &Types, id: TypeId) -> Self;
    fn gpr_list(&self) -> Vec<Reg>;
    fn arg_reg_list(&self, cc: &CallConvKind) -> Vec<Reg>;
    fn arg_reg_unit_list(&self, cc: &CallConvKind) -> Vec<RegUnit>;
    fn apply_for(&self, ru: RegUnit) -> Reg;
}

impl RegUnit {
    pub fn apply<RC: RegisterClass>(self, rc: &RC) -> Reg {
        rc.apply_for(self)
    }
}

impl VRegs {
    pub fn new() -> Self {
        Self {
            map: FxHashMap::default(),
            cur: 0,
        }
    }

    pub fn add_vreg_data(&mut self, ty: TypeId) -> VReg {
        let key = VReg(self.cur);
        self.map.insert(key, VRegData { vreg: key, ty });
        self.cur += 1;
        key
    }

    pub fn type_for(&self, vreg: VReg) -> TypeId {
        self.map[&vreg].ty
    }

    pub fn change_ty(&mut self, vreg: VReg, ty: TypeId) {
        self.map.get_mut(&vreg).unwrap().ty = ty
    }
}
