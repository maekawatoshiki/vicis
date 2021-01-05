use crate::codegen::target::Target;
use crate::ir::types::TypeId;
use id_arena::{Arena, Id};

pub type SlotId = Id<Slot>;

pub struct Slots<T: Target> {
    pub target: T,
    arena: Arena<Slot>,
}

#[derive(Debug, Clone)]
pub struct Slot {
    size: u32,
    ty: TypeId,
    num_elements: u32,
    align: u32,
}

impl<T: Target> Slots<T> {
    pub fn new(target: T) -> Self {
        Self {
            target,
            arena: Arena::new(),
        }
    }

    pub fn add_slot(&mut self, ty: TypeId) -> SlotId {
        self.arena.alloc(Slot {
            size: 1,
            ty,
            num_elements: 0,
            align: 0,
        })
    }

    pub fn unaligned_size(&self) -> u32 {
        let mut total = 0;
        for (_, slot) in &self.arena {
            total += slot.size;
        }
        total
    }
}
