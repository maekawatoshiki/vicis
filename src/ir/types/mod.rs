pub mod parser;

use id_arena::{Arena, Id};
use rustc_hash::FxHashMap;
use std::{
    cell::{Ref, RefCell, RefMut},
    fmt,
    sync::Arc,
};

pub use parser::parse;

pub type AddrSpace = u32;
pub type Cache<T> = FxHashMap<T, TypeId>;
pub type TypeId = Id<Type>;

#[derive(Clone)]
pub struct Types(Arc<RefCell<TypesBase>>);

pub struct TypesBase {
    arena: Arena<Type>,
    void: TypeId,
    int: Cache<u32>,
    pointer: Cache<(TypeId, u32)>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Type {
    Void,
    Int(u32),
    Pointer(PointerType),
    Function(FunctionType),
    // TODO: Add more types
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct PointerType {
    pub inner: TypeId,
    pub addr_space: AddrSpace,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct FunctionType {
    pub ret: TypeId,
    pub params: Vec<TypeId>,
    pub is_var_arg: bool,
}

impl Types {
    pub fn new() -> Self {
        Self(Arc::new(RefCell::new(TypesBase::new())))
    }

    pub fn base(&self) -> Ref<TypesBase> {
        self.0.borrow()
    }

    pub fn base_mut(&self) -> RefMut<TypesBase> {
        self.0.borrow_mut()
    }

    pub fn get(&self, id: TypeId) -> Ref<Type> {
        Ref::map(self.0.borrow(), |base| &base.arena[id])
    }

    pub fn get_mut(&mut self, id: TypeId) -> RefMut<Type> {
        RefMut::map(self.0.borrow_mut(), |base| &mut base.arena[id])
    }
}

impl TypesBase {
    pub fn new() -> Self {
        let mut arena = Arena::new();
        let void = arena.alloc(Type::Void);
        let int: Cache<u32> = vec![
            (1, Type::Int(1)),
            (8, Type::Int(8)),
            (16, Type::Int(16)),
            (32, Type::Int(32)),
            (64, Type::Int(64)),
        ]
        .into_iter()
        .map(|(bits, ty)| (bits, arena.alloc(ty)))
        .collect();
        Self {
            arena,
            void,
            int,
            pointer: Cache::default(),
        }
    }

    pub fn void(&self) -> TypeId {
        self.void.clone()
    }

    pub fn int(&mut self, bits: u32) -> TypeId {
        self.int
            .entry(bits)
            .or_insert(self.arena.alloc(Type::Int(bits)))
            .clone()
    }

    pub fn i1(&self) -> TypeId {
        self.int[&1].clone()
    }

    pub fn i8(&self) -> TypeId {
        self.int[&8].clone()
    }

    pub fn i16(&self) -> TypeId {
        self.int[&16].clone()
    }

    pub fn i32(&self) -> TypeId {
        self.int[&32].clone()
    }

    pub fn i64(&self) -> TypeId {
        self.int[&64].clone()
    }

    pub fn pointer(&mut self, inner: TypeId) -> TypeId {
        self.pointer
            .entry((inner, 0))
            .or_insert(self.arena.alloc(Type::Pointer(PointerType {
                inner,
                addr_space: 0,
            })))
            .clone()
    }

    pub fn pointer_in_addr_space(&mut self, inner: TypeId, addr_space: u32) -> TypeId {
        self.pointer
            .entry((inner, 0))
            .or_insert(
                self.arena
                    .alloc(Type::Pointer(PointerType { inner, addr_space })),
            )
            .clone()
    }
}

impl fmt::Debug for Types {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Types")
    }
}

#[test]
fn types_identity() {
    let types = Types::new();
    let i32_ptr_ty = {
        let i32_ty = types.base().i32();
        types.base_mut().pointer(i32_ty)
    };

    {
        let i32_ty = types.base().i32();
        let ty = types.get(i32_ptr_ty);
        assert_eq!(
            &*ty,
            &Type::Pointer(PointerType {
                inner: i32_ty,
                addr_space: 0
            })
        )
    }

    let i32_ty = types.base().i32();
    let i32_ptr_ty2 = types.base_mut().pointer(i32_ty);

    assert_eq!(i32_ptr_ty, i32_ptr_ty2);
}
