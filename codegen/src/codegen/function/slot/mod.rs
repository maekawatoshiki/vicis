use crate::codegen::isa::TargetIsa;
use id_arena::{Arena, Id};
use vicis_core::ir::types::Type;

pub type SlotId = Id<Slot>;

pub struct Slots<'a, T: TargetIsa> {
    pub isa: &'a T,
    arena: Arena<Slot>,
}

#[derive(Debug, Clone)]
pub struct Slot {
    pub(crate) size: u32,
    #[allow(dead_code)]
    pub(crate) ty: Type,
    #[allow(dead_code)]
    pub(crate) num_elements: u32,
    #[allow(dead_code)]
    pub(crate) align: u32,
}

impl<'a, T: TargetIsa> Slots<'a, T> {
    pub fn new(isa: &'a T) -> Self {
        Self {
            isa,
            arena: Arena::new(),
        }
    }

    pub fn add_slot(&mut self, ty: Type, size: u32) -> SlotId {
        self.arena.alloc(Slot {
            size,
            ty,
            num_elements: 0,
            align: 0,
        })
    }

    pub fn get(&self, id: SlotId) -> &Slot {
        &self.arena[id]
    }

    pub fn unaligned_size(&self) -> u32 {
        let mut total = 0;
        for (_, slot) in &self.arena {
            total += slot.size;
        }
        total
    }
}
