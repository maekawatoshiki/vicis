use crate::ir::types::TypeId;
use rustc_hash::FxHashMap;

#[derive(Debug, Clone, Copy)]
pub struct Reg(pub u16, pub u16);

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
}
