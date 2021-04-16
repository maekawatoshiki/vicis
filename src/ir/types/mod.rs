pub mod parser;

use crate::ir::module::name::Name;
use id_arena::{Arena, Id};
use rustc_hash::FxHashMap;
use std::{
    cell::{Ref, RefCell, RefMut},
    fmt, mem,
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
    named_types: Cache<Name>,
    void: TypeId,
    int: Cache<u32>,
    pointer: Cache<(TypeId, u32)>,
    array: Cache<(TypeId, u32)>,
    structs: Cache<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Void,
    Int(u32),
    Pointer(PointerType),
    Array(ArrayType),
    Function(FunctionType),
    Struct(StructType),
    // TODO: Add more types
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct PointerType {
    pub inner: TypeId,
    pub addr_space: AddrSpace,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct ArrayType {
    pub inner: TypeId,
    pub num_elements: u32,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct FunctionType {
    pub ret: TypeId,
    pub params: Vec<TypeId>,
    pub is_var_arg: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct StructType {
    pub name: Option<String>,
    pub elems: Vec<TypeId>,
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

    pub fn get_mut(&self, id: TypeId) -> RefMut<Type> {
        RefMut::map(self.0.borrow_mut(), |base| &mut base.arena[id])
    }

    // Get element type of pointer or array type
    pub fn get_element(&self, id: TypeId) -> Option<TypeId> {
        self.0.borrow().element(id)
    }

    pub fn to_string(&self, ty: TypeId) -> String {
        self.base().to_string(ty)
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
            named_types: Cache::default(),
            void,
            int,
            pointer: Cache::default(),
            array: Cache::default(),
            structs: Cache::default(),
        }
    }

    pub fn named_type(&mut self, name: Name) -> TypeId {
        *self
            .named_types
            .entry(name)
            .or_insert(self.arena.alloc(Type::Void))
    }

    pub fn void(&self) -> TypeId {
        self.void
    }

    pub fn int(&mut self, bits: u32) -> TypeId {
        *self
            .int
            .entry(bits)
            .or_insert(self.arena.alloc(Type::Int(bits)))
    }

    pub fn i1(&self) -> TypeId {
        self.int[&1]
    }

    pub fn i8(&self) -> TypeId {
        self.int[&8]
    }

    pub fn i16(&self) -> TypeId {
        self.int[&16]
    }

    pub fn i32(&self) -> TypeId {
        self.int[&32]
    }

    pub fn i64(&self) -> TypeId {
        self.int[&64]
    }

    pub fn pointer(&mut self, inner: TypeId) -> TypeId {
        *self
            .pointer
            .entry((inner, 0))
            .or_insert(self.arena.alloc(Type::Pointer(PointerType {
                inner,
                addr_space: 0,
            })))
    }

    pub fn pointer_in_addr_space(&mut self, inner: TypeId, addr_space: u32) -> TypeId {
        *self.pointer.entry((inner, 0)).or_insert(
            self.arena
                .alloc(Type::Pointer(PointerType { inner, addr_space })),
        )
    }

    pub fn array(&mut self, inner: TypeId, num_elements: u32) -> TypeId {
        *self
            .array
            .entry((inner, num_elements))
            .or_insert(self.arena.alloc(Type::Array(ArrayType {
                inner,
                num_elements,
            })))
    }

    pub fn function(&mut self, ret: TypeId, params: Vec<TypeId>, is_var_arg: bool) -> TypeId {
        // TODO: FIXME: Should cache function type?
        self.arena.alloc(Type::Function(FunctionType {
            ret,
            params,
            is_var_arg,
        }))
    }

    pub fn empty_struct_named(&mut self, name: String) -> TypeId {
        *self
            .structs
            .entry(name.clone())
            .or_insert(self.arena.alloc(Type::Struct(StructType {
                name: Some(name),
                elems: vec![],
            })))
    }

    pub fn anonymous_struct(&mut self, elems: Vec<TypeId>) -> TypeId {
        self.arena
            .alloc(Type::Struct(StructType { name: None, elems }))
    }

    pub fn get_struct(&self, name: &str) -> Option<TypeId> {
        self.structs.get(name).copied()
    }

    pub fn change_to_named_type(&mut self, ty: TypeId, name: Name) {
        let named_ty = self.named_type(name.clone());

        if self.is_struct(ty) {
            let mut strukt = mem::replace(&mut self.arena[ty], Type::Void);
            if let Some(name) = name.to_string() {
                strukt.as_struct_mut().name = Some(name.to_owned());
                self.structs.insert(name.to_owned(), named_ty);
            }
            self.arena[named_ty] = strukt;
            return;
        }

        let ty = self.arena[ty].clone();
        self.arena[named_ty] = ty;
    }

    pub fn element(&self, ty: TypeId) -> Option<TypeId> {
        match self.arena[ty] {
            Type::Void => None,
            Type::Int(_) => None,
            Type::Pointer(PointerType { inner, .. }) => Some(inner),
            Type::Array(ArrayType { inner, .. }) => Some(inner),
            Type::Function(_) => None,
            Type::Struct(_) => None,
        }
    }

    pub fn to_string(&self, ty: TypeId) -> String {
        let ty = &self.arena[ty];
        match ty {
            Type::Void => "void".to_string(),
            Type::Int(bits) => format!("i{}", bits),
            Type::Pointer(PointerType { inner, addr_space }) if *addr_space == 0 => {
                format!("{}*", self.to_string(*inner))
            }
            Type::Pointer(PointerType { inner, addr_space }) => {
                format!("{} addrspace({})*", self.to_string(*inner), addr_space)
            }
            Type::Array(ArrayType {
                inner,
                num_elements,
            }) => {
                format!("[{} x {}]", num_elements, self.to_string(*inner))
            }
            Type::Function(FunctionType {
                ret,
                params,
                is_var_arg,
            }) => {
                format!(
                    "{} ({})",
                    self.to_string(*ret),
                    params
                        .iter()
                        .enumerate()
                        .fold("".to_string(), |acc, (i, &param)| {
                            format!(
                                "{}{}{}",
                                acc,
                                self.to_string(param),
                                if i == params.len() - 1 {
                                    if *is_var_arg {
                                        ", ..."
                                    } else {
                                        ""
                                    }
                                } else {
                                    ", "
                                }
                            )
                        })
                )
            }
            Type::Struct(ty) => {
                if let Some(name) = ty.name.as_ref() {
                    return format!("%{}", name);
                }
                self.struct_definition_to_string(ty)
            }
        }
    }

    pub fn struct_definition_to_string(&self, ty: &StructType) -> String {
        let mut elems_str = "".to_string();
        for (i, elem) in ty.elems.iter().enumerate() {
            elems_str.push_str(self.to_string(*elem).as_str());
            if i != ty.elems.len() - 1 {
                elems_str.push_str(", ");
            }
        }
        format!("{{ {} }}", elems_str)
    }

    pub fn is_struct(&self, ty: TypeId) -> bool {
        matches!(self.arena[ty], Type::Struct(_))
    }
}

impl Type {
    pub fn as_struct(&self) -> &StructType {
        match self {
            Self::Struct(strct) => strct,
            _ => panic!(),
        }
    }

    pub fn as_struct_mut(&mut self) -> &mut StructType {
        match self {
            Self::Struct(strct) => strct,
            _ => panic!(),
        }
    }
}

impl fmt::Debug for Types {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (name, &id) in &self.base().named_types {
            writeln!(
                f,
                "%{} = type {}",
                name,
                match &*self.get(id) {
                    Type::Struct(ty) => {
                        self.base().struct_definition_to_string(ty)
                    }
                    _ => self.to_string(id),
                }
            )?
        }
        Ok(())
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
