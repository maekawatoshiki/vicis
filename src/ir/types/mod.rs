use rustc_hash::FxHashMap;
use std::sync::Arc;

pub type TypeRef = Arc<Type>;
pub type AddrSpace = u32;
pub type Cache<T> = FxHashMap<T, TypeRef>;

pub struct Types {
    void: TypeRef,
    int: Cache<u32>,
}

pub enum Type {
    Void,
    Int(u32),
    Pointer(PointerType),
    Function(FunctionType),
    // TODO: Add more types
}

pub struct PointerType {
    pub inner: TypeRef,
    pub addr_space: AddrSpace,
}

pub struct FunctionType {
    pub ret: TypeRef,
    pub params: Vec<TypeRef>,
    pub var_arg: bool,
}

impl Types {
    pub fn new() -> Self {
        Self {
            void: TypeRef::new(Type::Void),
            int: vec![
                (1, TypeRef::new(Type::Int(1))),
                (8, TypeRef::new(Type::Int(8))),
                (16, TypeRef::new(Type::Int(16))),
                (32, TypeRef::new(Type::Int(32))),
                (64, TypeRef::new(Type::Int(64))),
            ]
            .into_iter()
            .collect(),
        }
    }

    pub fn void(&self) -> TypeRef {
        self.void.clone()
    }

    pub fn int(&mut self, bits: u32) -> TypeRef {
        self.int
            .entry(bits)
            .or_insert(TypeRef::new(Type::Int(bits)))
            .clone()
    }

    pub fn i1(&self) -> TypeRef {
        self.int[&1].clone()
    }

    pub fn i8(&self) -> TypeRef {
        self.int[&8].clone()
    }

    pub fn i16(&self) -> TypeRef {
        self.int[&16].clone()
    }

    pub fn i32(&self) -> TypeRef {
        self.int[&32].clone()
    }

    pub fn i64(&self) -> TypeRef {
        self.int[&64].clone()
    }
}
