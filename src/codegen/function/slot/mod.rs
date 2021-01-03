use crate::codegen::target::Target;
use std::marker::PhantomData;

pub struct Slots<T: Target> {
    phantom: PhantomData<T>,
}

impl<T: Target> Slots<T> {
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}
