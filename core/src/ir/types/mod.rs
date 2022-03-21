use crate::ir::module::name::Name;
use rustc_hash::FxHashMap;
use std::{
    cell::{Ref, RefCell, RefMut},
    fmt, mem,
    sync::{atomic, atomic::AtomicU32, Arc},
};

pub type AddrSpace = u32;
type Idx = u32;
type Cache<T> = FxHashMap<T, Type>;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Type(Idx, Idx); // (arena id, type id)

pub const VOID: Type = Type(0, 0);
pub const I1: Type = Type(0, 1);
pub const I8: Type = Type(0, 2);
pub const I16: Type = Type(0, 3);
pub const I32: Type = Type(0, 4);
pub const I64: Type = Type(0, 5);

/// Represents a typed value.
pub trait Typed {
    fn ty(&self) -> Type;
}

#[derive(Clone)]
pub struct Types(Arc<RefCell<TypesBase>>);

pub struct TypesBase {
    arena_id: Idx,
    id: Idx,
    compound_types: Vec<CompoundType>,
    caches: Caches,
}

#[derive(Debug)]
struct Caches {
    pointer: Cache<PointerType>,
    array: Cache<ArrayType>,
    named_struct: Cache<String>,
    named_types: Cache<Name>,
    metadata: Type,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum CompoundType {
    Pointer(PointerType),
    Array(ArrayType),
    Function(FunctionType),
    Struct(StructType),
    Alias(Type),
    Metadata,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct PointerType {
    pub inner: Type,
    pub addr_space: AddrSpace,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct ArrayType {
    pub inner: Type,
    pub num_elements: u32,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct FunctionType {
    pub ret: Type,
    pub params: Vec<Type>,
    pub is_var_arg: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, Default)]
pub struct StructType {
    pub name: Option<Name>,
    pub elems: Vec<Type>,
    pub is_packed: bool,
}

impl Default for Types {
    fn default() -> Self {
        Self(Arc::new(RefCell::new(TypesBase::new())))
    }
}

impl Types {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn to_string(&self, ty: Type) -> String {
        self.base().to_string(ty)
    }

    pub fn get(&self, ty: Type) -> Option<Ref<CompoundType>> {
        if ty.is_primitive() {
            return None;
        }
        Some(Ref::map(self.0.borrow(), |base| base.get(ty).unwrap()))
    }

    pub fn get_mut(&self, ty: Type) -> Option<RefMut<CompoundType>> {
        if ty.is_primitive() {
            return None;
        }
        Some(RefMut::map(self.0.borrow_mut(), |base| {
            base.get_mut(ty).unwrap()
        }))
    }

    pub fn get_element(&self, ty: Type) -> Option<Type> {
        self.base().element(ty)
    }

    pub fn base(&self) -> Ref<TypesBase> {
        self.0.borrow()
    }

    pub fn base_mut(&self) -> RefMut<TypesBase> {
        self.0.borrow_mut()
    }

    pub fn is_pointer(&self, ty: Type) -> bool {
        self.base().is_pointer(ty)
    }

    pub fn is_array(&self, ty: Type) -> bool {
        self.base().is_array(ty)
    }

    pub fn is_struct(&self, ty: Type) -> bool {
        self.base().is_struct(ty)
    }

    pub fn metadata(&self) -> Type {
        self.base().caches.metadata
    }
}

impl Default for TypesBase {
    fn default() -> Self {
        static ID: AtomicU32 = AtomicU32::new(1); // 0 is reserved for primitive types
        let arena_id = ID.fetch_add(1, atomic::Ordering::SeqCst);
        Self {
            arena_id,
            id: 1,
            compound_types: vec![CompoundType::Metadata],
            caches: Caches {
                metadata: Type(arena_id, 0),
                pointer: Cache::default(),
                array: Cache::default(),
                named_struct: Cache::default(),
                named_types: Cache::default(),
            },
        }
    }
}

impl TypesBase {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, ty: Type) -> Option<&CompoundType> {
        if ty.is_primitive() {
            return None;
        }
        Some(&self.compound_types[ty.1 as usize])
    }

    pub fn get_mut(&mut self, ty: Type) -> Option<&mut CompoundType> {
        if ty.is_primitive() {
            return None;
        }
        Some(&mut self.compound_types[ty.1 as usize])
    }

    pub fn is_pointer(&self, ty: Type) -> bool {
        matches!(self.get(ty), Some(CompoundType::Pointer(_)))
    }

    pub fn is_array(&self, ty: Type) -> bool {
        matches!(self.get(ty), Some(CompoundType::Array(_)))
    }

    pub fn new_type(&mut self, ty: CompoundType) -> Type {
        let id = self.id;
        self.id += 1;
        self.compound_types.push(ty);
        Type(self.arena_id, id)
    }

    pub fn empty_named_type(&mut self, name: Name) -> Type {
        if let Some(ty) = self.caches.named_types.get(&name) {
            return *ty;
        }
        let ty = self.new_type(CompoundType::Alias(VOID));
        self.caches.named_types.insert(name, ty);
        ty
    }

    pub fn pointer(&mut self, t: impl Into<PointerType>) -> Type {
        let t = t.into();
        if let Some(ty) = self.caches.pointer.get(&t) {
            return *ty;
        }
        let ty = self.new_type(CompoundType::Pointer(t.clone()));
        self.caches.pointer.insert(t, ty);
        ty
    }

    pub fn array(&mut self, t: ArrayType) -> Type {
        if let Some(ty) = self.caches.array.get(&t) {
            return *ty;
        }
        let ty = self.new_type(CompoundType::Array(t.clone()));
        self.caches.array.insert(t, ty);
        ty
    }

    pub fn function(&mut self, t: FunctionType) -> Type {
        // TODO: FIXME: Should cache function type?
        self.new_type(CompoundType::Function(t))
    }

    pub fn metadata(&mut self) -> Type {
        self.caches.metadata
    }

    pub fn empty_struct_named(&mut self, name: String, is_packed: bool) -> Type {
        if let Some(ty) = self.caches.named_struct.get(&name) {
            return *ty;
        }
        let ty = self.new_type(CompoundType::Struct(StructType {
            name: Some(Name::Name(name.clone())),
            elems: vec![],
            is_packed,
        }));
        self.caches.named_struct.insert(name, ty);
        ty
    }

    pub fn anonymous_struct(&mut self, elems: Vec<Type>, is_packed: bool) -> Type {
        self.new_type(CompoundType::Struct(StructType {
            name: None,
            elems,
            is_packed,
        }))
    }

    pub fn get_struct(&self, name: impl AsRef<str>) -> Option<Type> {
        self.caches.named_struct.get(name.as_ref()).copied()
    }

    pub fn is_struct(&self, ty: Type) -> bool {
        if ty.0 != self.arena_id {
            return false;
        }
        matches!(self.compound_types[ty.1 as usize], CompoundType::Struct(_))
    }

    pub fn change_to_named_type(&mut self, ty: Type, name: Name) {
        let named_ty = self.empty_named_type(name.clone());

        match self.get_mut(ty) {
            // primitive types
            None if ty.is_primitive() => {
                self.compound_types[named_ty.1 as usize] = CompoundType::Alias(ty);
            }
            // If `ty` is a struct type, name it.
            Some(CompoundType::Struct(ref mut strukt)) => {
                let mut strukt = mem::take(strukt);
                strukt.name = Some(name.clone());
                if let Name::Name(name) = name {
                    self.caches.named_struct.insert(name, named_ty);
                }
                self.compound_types[named_ty.1 as usize] = CompoundType::Struct(strukt);
            }
            _ => todo!(),
        }
    }

    pub fn element(&self, ty: Type) -> Option<Type> {
        match self.get(ty)? {
            CompoundType::Pointer(PointerType { inner, .. }) => Some(*inner),
            CompoundType::Array(ArrayType { inner, .. }) => Some(*inner),
            CompoundType::Struct(_) => None,
            CompoundType::Function(_) => None,
            CompoundType::Alias(t) => self.element(*t),
            CompoundType::Metadata => None,
        }
    }

    pub fn element_at(&self, ty: Type, i: usize) -> Option<Type> {
        match self.get(ty)? {
            CompoundType::Pointer(PointerType { inner, .. }) => Some(*inner),
            CompoundType::Array(ArrayType { inner, .. }) => Some(*inner),
            CompoundType::Struct(StructType { elems, .. }) => elems.get(i).copied(),
            CompoundType::Function(_) => None,
            CompoundType::Alias(t) => self.element_at(*t, i),
            CompoundType::Metadata => None,
        }
    }

    pub fn element_at_(&self, ty: Type, indices: impl Iterator<Item = usize>) -> Option<Type> {
        let mut ty = ty;
        for i in indices {
            match self.element_at(ty, i) {
                Some(t) => ty = t,
                None => return None,
            }
        }
        Some(ty)
    }

    pub fn to_string(&self, ty: Type) -> String {
        if ty.is_primitive() {
            return ty.to_string();
        }

        let ty = &self.get(ty).expect("must be compound type");
        match ty {
            CompoundType::Pointer(PointerType { inner, addr_space }) if *addr_space == 0 => {
                format!("{}*", self.to_string(*inner))
            }
            CompoundType::Pointer(PointerType { inner, addr_space }) => {
                format!("{} addrspace({})*", self.to_string(*inner), addr_space)
            }
            CompoundType::Array(ArrayType {
                inner,
                num_elements,
            }) => {
                format!("[{} x {}]", num_elements, self.to_string(*inner))
            }
            CompoundType::Function(FunctionType {
                ret,
                params,
                is_var_arg,
            }) => {
                format!(
                    "{} ({}{})",
                    self.to_string(*ret),
                    params
                        .iter()
                        .enumerate()
                        .fold("".to_string(), |acc, (i, &param)| {
                            format!(
                                "{}{}{}",
                                acc,
                                self.to_string(param),
                                if i == params.len() - 1 { "" } else { ", " }
                            )
                        }),
                    if *is_var_arg && params.is_empty() {
                        "..."
                    } else if *is_var_arg {
                        ", ..."
                    } else {
                        ""
                    }
                )
            }
            CompoundType::Struct(ty) => {
                if let Some(name) = ty.name.as_ref() {
                    return format!("%{}", name);
                }
                self.struct_definition_to_string(ty)
            }
            CompoundType::Metadata => "metadata".to_string(),
            CompoundType::Alias(t) => t.to_string(),
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
        format!(
            "{}{{ {} }}{}",
            if ty.is_packed { "<" } else { "" },
            elems_str,
            if ty.is_packed { ">" } else { "" }
        )
    }
}

impl Type {
    pub fn is_primitive(&self) -> bool {
        self.0 == 0
    }

    pub fn is_void(&self) -> bool {
        self == &VOID
    }

    pub fn is_i1(&self) -> bool {
        self == &I1
    }

    pub fn is_i8(&self) -> bool {
        self == &I8
    }

    pub fn is_i16(&self) -> bool {
        self == &I16
    }

    pub fn is_i32(&self) -> bool {
        self == &I32
    }

    pub fn is_i64(&self) -> bool {
        self == &I64
    }

    pub fn is_pointer(&self, types: &Types) -> bool {
        types.is_pointer(*self)
    }

    pub fn is_array(&self, types: &Types) -> bool {
        types.is_array(*self)
    }

    pub fn is_struct(&self, types: &Types) -> bool {
        types.is_struct(*self)
    }
}

impl ArrayType {
    pub fn new(inner: Type, num_elements: u32) -> Self {
        Self {
            inner,
            num_elements,
        }
    }
}

impl FunctionType {
    pub fn new(ret: Type, params: Vec<Type>, is_var_arg: bool) -> Self {
        Self {
            ret,
            params,
            is_var_arg,
        }
    }
}

impl ToString for Type {
    fn to_string(&self) -> String {
        if self.is_primitive() {
            return match *self {
                VOID => "void".to_string(),
                I1 => "i1".to_string(),
                I8 => "i8".to_string(),
                I16 => "i16".to_string(),
                I32 => "i32".to_string(),
                I64 => "i64".to_string(),
                _ => todo!(),
            };
        }
        format!("{:?}", self)
    }
}

impl From<Type> for PointerType {
    fn from(t: Type) -> Self {
        PointerType {
            inner: t,
            addr_space: 0,
        }
    }
}

impl fmt::Debug for Types {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (name, &ty) in &self.base().caches.named_types {
            writeln!(
                f,
                "%{} = type {}",
                name,
                match self.base().get(ty) {
                    Some(CompoundType::Struct(ty)) => self.base().struct_definition_to_string(ty),
                    _ => self.to_string(ty),
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
        let i32_ty = I32;
        types.base_mut().pointer(i32_ty)
    };

    {
        let i32_ty = I32;
        let ty = types.get(i32_ptr_ty);
        assert_eq!(
            &*ty.unwrap(),
            &CompoundType::Pointer(PointerType {
                inner: i32_ty,
                addr_space: 0
            })
        )
    }

    let i32_ty = I32;
    let i32_ptr_ty2 = types.base_mut().pointer(i32_ty);

    assert_eq!(i32_ptr_ty, i32_ptr_ty2);
}
