use crate::ir::{
    module::name::Name,
    types::{Type, Typed},
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ArgumentValue {
    pub nth: usize,
    pub ty: Type,
    pub name: Option<Name>,
}

impl ArgumentValue {
    pub fn new(nth: usize, ty: Type, name: Option<Name>) -> Self {
        Self { nth, ty, name }
    }
}

impl Typed for ArgumentValue {
    fn ty(&self) -> Type {
        self.ty
    }
}
