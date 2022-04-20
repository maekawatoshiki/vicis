use crate::isa::TargetIsa;
use id_arena::{Arena, Id};
use vicis_core::ir::types::Type;

pub type SlotId = Id<Slot>;

pub struct Slots<'a, T: TargetIsa> {
    pub isa: &'a T,
    arena: Arena<Slot>,
    aligned_size: u32,
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
    pub offset: u32,
}

impl<'a, T: TargetIsa> Slots<'a, T> {
    pub fn new(isa: &'a T) -> Self {
        Self {
            isa,
            arena: Arena::new(),
            aligned_size: 0,
        }
    }

    pub fn add_slot(&mut self, ty: Type, size: u32, align: u32) -> SlotId {
        self.aligned_size = 0;
        self.arena.alloc(Slot {
            size,
            ty,
            num_elements: 0,
            align,
            offset: 0,
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

    /// Makes sure that the offsets for the slots are computed.
    /// Returns the aligned total size of the slots.
    pub fn ensure_computed_offsets(&mut self) -> u32 {
        if self.aligned_size != 0 {
            return self.aligned_size;
        }

        let mut offset = 0;
        let mut align = 1;
        for (_id, slot) in &mut self.arena {
            if !is_aligned(slot.align, offset) {
                offset = align_to(offset, slot.align);
            }
            align = align.max(slot.align);
            offset += slot.size;
            slot.offset = offset;
        }
        if !is_aligned(align, offset) {
            offset = align_to(offset, align);
        }
        offset
    }
}

fn is_aligned(align: u32, offset: u32) -> bool {
    offset % align == 0
}

fn align_to(offset: u32, align: u32) -> u32 {
    (offset + align - 1) & !(align - 1)
}
