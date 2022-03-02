use crate::ir::types::{self, CompoundType, Type, Types};

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct DataLayout(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructLayout(Vec<usize>);

impl DataLayout {
    pub fn new_struct_layout_for(&self, types: &Types, ty: Type) -> StructLayout {
        StructLayout::new(self, types, ty)
    }

    /// Returns the size of the given type in bytes.
    pub fn get_size_of(&self, types: &Types, ty: Type) -> usize {
        fn align(n: usize, m: usize) -> usize {
            let rem = n % m;
            if rem == 0 {
                n
            } else {
                n - rem + m
            }
        }

        match types.get(ty) {
            Some(ty) => match &*ty {
                CompoundType::Array(a) => {
                    self.get_size_of(types, a.inner) * a.num_elements as usize
                }
                CompoundType::Pointer(_) => 8,
                CompoundType::Struct(s) => {
                    // TODO: This doesn't work correctly. Need to implement StructLayout.
                    if s.is_packed {
                        s.elems.iter().map(|&e| self.get_size_of(types, e)).sum()
                    } else {
                        align(s.elems.iter().map(|&e| self.get_size_of(types, e)).sum(), 8)
                    }
                }
                _ => todo!(),
            },
            None => match ty {
                types::VOID => return 0,
                types::I1 => return 1,
                types::I8 => return 1,
                types::I16 => return 2,
                types::I32 => return 4,
                types::I64 => return 8,
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
    pub fn new(_dl: &DataLayout, _types: &Types, _ty: Type) -> Self {
        // let mut size = 0;
        todo!()
    }

    pub fn get_elem_offset(&self, idx: usize) -> Option<usize> {
        self.0.get(idx).copied()
    }
}

#[test]
fn size() {
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
}
