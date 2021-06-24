use crate::{
    codegen::{
        call_conv::CallConvKind,
        function::instruction::{InstructionData as ID, InstructionId},
    },
    ir::types::{TypeId, Types},
};
use rustc_hash::FxHashMap;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub struct Reg(pub u16, pub u16);

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub struct RegUnit(pub u16, pub u16); // Same as top-level register

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

pub struct VRegUsers<Data: ID> {
    pub vreg_to_insts: FxHashMap<VReg, Vec<VRegUser<Data>>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VRegUser<Data: ID> {
    pub inst_id: InstructionId<Data>,
    pub read: bool,
    pub write: bool,
}

pub trait RegisterInfo {
    fn arg_reg_list(cc: &CallConvKind) -> &'static [RegUnit];
    fn to_reg_unit(reg: Reg) -> RegUnit;
}

pub trait RegisterClass {
    fn for_type(types: &Types, id: TypeId) -> Self;
    fn gpr_list(&self) -> Vec<Reg>;
    fn apply_for(&self, ru: RegUnit) -> Reg;
}

impl RegUnit {
    pub fn apply<RC: RegisterClass>(self, rc: &RC) -> Reg {
        rc.apply_for(self)
    }
}

impl Default for VRegs {
    fn default() -> Self {
        Self {
            map: FxHashMap::default(),
            cur: 0,
        }
    }
}

impl VRegs {
    pub fn new() -> Self {
        Self::default()
    }

    // TODO: Change name
    pub fn add_vreg_data(&mut self, ty: TypeId) -> VReg {
        let key = VReg(self.cur);
        self.map.insert(key, VRegData { vreg: key, ty });
        self.cur += 1;
        key
    }

    pub fn create_from(&mut self, vreg: VReg) -> VReg {
        let ty = self.map[&vreg].ty;
        self.add_vreg_data(ty)
    }

    pub fn type_for(&self, vreg: VReg) -> TypeId {
        self.map[&vreg].ty
    }

    pub fn change_ty(&mut self, vreg: VReg, ty: TypeId) {
        self.map.get_mut(&vreg).unwrap().ty = ty
    }
}

impl<Data: ID> Default for VRegUsers<Data> {
    fn default() -> Self {
        Self {
            vreg_to_insts: FxHashMap::default(),
        }
    }
}

impl<Data: ID> VRegUsers<Data> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_use(&mut self, vreg: VReg, inst_id: InstructionId<Data>, read: bool, write: bool) {
        self.vreg_to_insts
            .entry(vreg)
            .or_insert_with(Vec::new)
            .push(VRegUser {
                inst_id,
                read,
                write,
            })
    }

    pub fn get(&self, vreg: VReg) -> &Vec<VRegUser<Data>> {
        &self.vreg_to_insts[&vreg]
    }

    pub fn remove_use(
        &mut self,
        vreg: VReg,
        inst_id: InstructionId<Data>,
    ) -> Option<VRegUser<Data>> {
        if let Some(idx) = self.vreg_to_insts[&vreg]
            .iter()
            .position(|u| u.inst_id == inst_id)
        {
            return Some(self.vreg_to_insts.get_mut(&vreg).unwrap().remove(idx));
        }
        None
    }
}
