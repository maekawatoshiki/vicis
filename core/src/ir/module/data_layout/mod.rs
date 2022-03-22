use crate::ir::types::{self, CompoundType, StructType, Type, Types};

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct DataLayout(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructLayout(Vec<usize>, usize, usize);

impl DataLayout {
    pub fn new_struct_layout_for(&self, types: &Types, ty: Type) -> Option<StructLayout> {
        if let CompoundType::Struct(ref sty) = &*types.get(ty)? {
            return Some(StructLayout::new(self, types, sty));
        }
        None
    }

    /// Returns the size of the given type in bytes.
    pub fn get_size_of(&self, types: &Types, ty: Type) -> usize {
        match types.get(ty) {
            Some(ty) => match &*ty {
                CompoundType::Array(a) => {
                    self.get_size_of(types, a.inner) * a.num_elements as usize
                }
                CompoundType::Pointer(_) => 8,
                CompoundType::Struct(s) => StructLayout::new(self, types, s).get_size(),
                e => todo!("{:?}", e),
            },
            None => match ty {
                types::VOID => 0,
                types::I1 => 1,
                types::I8 => 1,
                types::I16 => 2,
                types::I32 => 4,
                types::I64 => 8,
                x => todo!("sizeof {:?}", x),
            },
        }
    }

    pub fn get_align_of(&self, types: &Types, ty: Type) -> usize {
        match types.get(ty) {
            Some(ty) => match &*ty {
                CompoundType::Array(a) => self.get_size_of(types, a.inner),
                CompoundType::Pointer(_) => 8,
                CompoundType::Struct(s) => StructLayout::new(self, types, s).get_align(),
                _ => todo!(),
            },
            None => match ty {
                types::VOID => 0,
                types::I1 => 1,
                types::I8 => 1,
                types::I16 => 2,
                types::I32 => 4,
                types::I64 => 8,
                x => todo!("sizeof {:?}", x),
            },
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for DataLayout {
    fn from(s: String) -> Self {
        DataLayout(s)
    }
}

impl StructLayout {
    pub fn new(dl: &DataLayout, types: &Types, sty: &StructType) -> Self {
        let mut offset = 0;
        let mut align = 1;
        let mut offsets = vec![];
        for &elem in &sty.elems {
            let elem_align = if sty.is_packed {
                1
            } else {
                dl.get_align_of(types, elem)
            };
            if !is_aligned(elem_align, offset) {
                offset = align_to(offset, elem_align);
            }
            align = align.max(elem_align);
            offsets.push(offset);
            offset += dl.get_size_of(types, elem);
        }
        if !is_aligned(align, offset) {
            offset = align_to(offset, align);
        }
        Self(offsets, offset, align)
    }

    pub fn get_elem_offset(&self, idx: usize) -> Option<usize> {
        self.0.get(idx).copied()
    }

    pub fn get_size(&self) -> usize {
        self.1
    }

    pub fn get_align(&self) -> usize {
        self.2
    }
}

fn is_aligned(align: usize, offset: usize) -> bool {
    offset % align == 0
}

fn align_to(offset: usize, align: usize) -> usize {
    (offset + align - 1) & !(align - 1)
}

#[test]
fn size() {
    use crate::ir::types::*;

    let dl = DataLayout("".to_string());
    let types = Types::new();
    assert_eq!(dl.get_size_of(&types, types::VOID), 0);
    assert_eq!(dl.get_size_of(&types, types::I1), 1);
    assert_eq!(dl.get_size_of(&types, types::I8), 1);
    assert_eq!(dl.get_size_of(&types, types::I16), 2);
    assert_eq!(dl.get_size_of(&types, types::I32), 4);
    assert_eq!(dl.get_size_of(&types, types::I64), 8);

    let i8_ptr = types.base_mut().pointer(types::I8);
    assert_eq!(dl.get_size_of(&types, i8_ptr), 8);

    let i8_arr = types.base_mut().array(ArrayType::new(types::I8, 100));
    assert_eq!(dl.get_size_of(&types, i8_arr), 100);

    let i32_arr = types.base_mut().array(ArrayType::new(types::I32, 100));
    assert_eq!(dl.get_size_of(&types, i32_arr), 400);

    let strukt0 = types
        .base_mut()
        .anonymous_struct(vec![types::I32, types::I32], false);
    assert_eq!(dl.get_size_of(&types, strukt0), 8);
    let strukt1 = types
        .base_mut()
        .anonymous_struct(vec![types::I64, types::I64, types::I64], false);
    assert_eq!(dl.get_size_of(&types, strukt1), 24);
    let strukt2 = types
        .base_mut()
        .anonymous_struct(vec![types::I64, types::I64, types::I32], false);
    assert_eq!(dl.get_size_of(&types, strukt2), 24);
    let strukt3 = types
        .base_mut()
        .anonymous_struct(vec![types::I8, strukt2, types::I32], false);
    assert_eq!(dl.get_size_of(&types, strukt3), 40);
}

#[test]
fn align() {
    use crate::ir::types::*;

    let dl = DataLayout("".to_string());
    let types = Types::new();
    assert_eq!(dl.get_align_of(&types, types::VOID), 0);
    assert_eq!(dl.get_align_of(&types, types::I1), 1);
    assert_eq!(dl.get_align_of(&types, types::I8), 1);
    assert_eq!(dl.get_align_of(&types, types::I16), 2);
    assert_eq!(dl.get_align_of(&types, types::I32), 4);
    assert_eq!(dl.get_align_of(&types, types::I64), 8);

    let i8_ptr = types.base_mut().pointer(types::I8);
    assert_eq!(dl.get_align_of(&types, i8_ptr), 8);

    let i8_arr = types.base_mut().array(ArrayType::new(types::I8, 100));
    assert_eq!(dl.get_align_of(&types, i8_arr), 1);

    let i32_arr = types.base_mut().array(ArrayType::new(types::I32, 100));
    assert_eq!(dl.get_align_of(&types, i32_arr), 4);

    let strukt0 = types
        .base_mut()
        .anonymous_struct(vec![types::I32, types::I32], false);
    assert_eq!(dl.get_align_of(&types, strukt0), 4);
    let strukt1 = types
        .base_mut()
        .anonymous_struct(vec![types::I64, types::I64, types::I64], false);
    assert_eq!(dl.get_align_of(&types, strukt1), 8);
    let strukt2 = types
        .base_mut()
        .anonymous_struct(vec![types::I64, types::I64, types::I32], false);
    assert_eq!(dl.get_align_of(&types, strukt2), 8);
    let strukt3 = types
        .base_mut()
        .anonymous_struct(vec![types::I8, strukt2, types::I32], false);
    assert_eq!(dl.get_align_of(&types, strukt3), 8);
}
