use super::generic_value::GenericValue;
use crate::ir::{
    function::FunctionId,
    instruction::Operand,
    module::Module,
    types::{Type, TypeId, Types},
};

pub struct Interpreter<'a> {
    module: &'a Module,
}

impl<'a> Interpreter<'a> {
    pub fn new(module: &'a Module) -> Self {
        Self { module }
    }

    pub fn run_function(&mut self, func_id: FunctionId) -> Option<GenericValue> {
        let func = &self.module.functions()[func_id];
        let mut alloca_size = 0;
        let mut mem: Vec<u8> = vec![];

        let mut block = func.layout.first_block?;

        loop {
            for inst in func
                .layout
                .inst_iter(block)
                .into_iter()
                .map(|id| func.data.inst_ref(id))
            {
                match &inst.operand {
                    Operand::Alloca {
                        tys,
                        num_elements,
                        align: _,
                    } => {
                        let alloc_ty = tys[0];
                        let alloc_sz = self.module.types.size_of(alloc_ty)
                            * num_elements.as_int().cast_to_usize();
                        alloca_size += alloc_sz;
                        mem.resize(alloca_size, 0);
                    }
                    _ => {}
                }
            }

            if let Some(next) = func.layout.next_block_of(block) {
                block = next;
                continue;
            }

            break;
        }

        println!("alloca size: {}", alloca_size);

        Some(GenericValue::Int32(0))
    }
}

// dummy

trait TypeSize {
    fn size_of(&self, ty: TypeId) -> usize;
}

impl TypeSize for Types {
    // Returns the size of the type in byte
    fn size_of(&self, ty: TypeId) -> usize {
        let ty = self.get(ty);
        match &*ty {
            Type::Void => 0,
            Type::Int(1) => 1,
            Type::Int(8) => 1,
            Type::Int(16) => 2,
            Type::Int(32) => 4,
            _ => todo!(),
        }
    }
}
