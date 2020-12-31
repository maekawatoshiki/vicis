use super::generic_value::GenericValue;
use crate::ir::{
    function::FunctionId,
    instruction::Operand,
    module::Module,
    types::{Type, TypeId, Types},
    value::{ConstantData, ConstantInt, Value},
};
use rustc_hash::FxHashMap;

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
        let mut mem: Vec<u8> = Vec::with_capacity(1024);
        mem.resize(1024, 0);
        let mem = mem.into_raw_parts().0;
        let mut id_to_genvalue = FxHashMap::default();

        let mut block = func.layout.first_block?;

        loop {
            for (inst_id, inst) in func
                .layout
                .inst_iter(block)
                .into_iter()
                .map(|id| (id, func.data.inst_ref(id)))
            {
                match &inst.operand {
                    Operand::Alloca {
                        tys,
                        num_elements,
                        align: _,
                    } => {
                        id_to_genvalue
                            .insert(inst_id, GenericValue::Ptr(unsafe { mem.add(alloca_size) }));
                        let alloc_ty = tys[0];
                        let alloc_sz = self.module.types.size_of(alloc_ty)
                            * num_elements.as_int().cast_to_usize();
                        alloca_size += alloc_sz;
                    }
                    Operand::Store {
                        tys: _,
                        args,
                        align: _,
                    } => {
                        let src = args[0];
                        let dst = args[1];
                        let dst_addr = match func.data.value_ref(dst) {
                            Value::Instruction(id) => id_to_genvalue[id],
                            _ => todo!(),
                        };
                        match func.data.value_ref(src) {
                            Value::Constant(ConstantData::Int(ConstantInt::Int32(i))) => unsafe {
                                *(dst_addr.as_ptr().unwrap() as *mut i32) = *i;
                            },
                            _ => todo!(),
                        }
                    }
                    Operand::Load {
                        tys,
                        addr,
                        align: _,
                    } => {
                        let ty = tys[0];
                        let addr = match func.data.value_ref(*addr) {
                            Value::Instruction(id) => id_to_genvalue[id],
                            _ => todo!(),
                        };
                        match &*func.types.get(ty) {
                            Type::Int(32) => id_to_genvalue.insert(
                                inst_id,
                                GenericValue::Int32(unsafe {
                                    *(addr.as_ptr().unwrap() as *const i32)
                                }),
                            ),
                            _ => todo!(),
                        };
                    }
                    Operand::Ret { val, .. } if val.is_none() => return Some(GenericValue::Void),
                    Operand::Ret {
                        ty: _,
                        val: Some(val),
                    } => match func.data.value_ref(*val) {
                        Value::Instruction(id) => return Some(id_to_genvalue[id]),
                        _ => todo!(),
                    },
                    _ => todo!(),
                }
            }

            if let Some(next) = func.layout.next_block_of(block) {
                block = next;
                continue;
            }

            break;
        }

        panic!("reached end of function without terminator");
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
