use super::generic_value::GenericValue;
use crate::ir::{
    function::{Data, FunctionId},
    instruction::{ICmpCond, InstructionId, Opcode, Operand},
    module::Module,
    types::{Type, TypeId, Types},
    value::{ConstantData, ConstantInt, Value, ValueId},
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

        'outer: loop {
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
                        tys,
                        args,
                        align: _,
                    } => {
                        let ty = tys[0];
                        let src = args[0];
                        let dst = args[1];
                        let dst_addr = genvalue(&func.data, &id_to_genvalue, dst);
                        match func.data.value_ref(src) {
                            Value::Constant(ConstantData::Int(ConstantInt::Int32(i))) => unsafe {
                                *(dst_addr.to_ptr().unwrap() as *mut i32) = *i;
                            },
                            Value::Instruction(id)
                                if matches!(&*func.types.get(ty), Type::Int(32)) =>
                            unsafe {
                                *(dst_addr.to_ptr().unwrap() as *mut i32) =
                                    id_to_genvalue[id].to_i32().unwrap();
                            }
                            e => todo!("{:?}", e),
                        }
                    }
                    Operand::Load {
                        tys,
                        addr,
                        align: _,
                    } => {
                        let ty = tys[0];
                        let addr = genvalue(&func.data, &id_to_genvalue, *addr);
                        match &*func.types.get(ty) {
                            Type::Int(32) => id_to_genvalue.insert(
                                inst_id,
                                GenericValue::Int32(unsafe {
                                    *(addr.to_ptr().unwrap() as *const i32)
                                }),
                            ),
                            _ => todo!(),
                        };
                    }
                    Operand::IntBinary {
                        ty: _,
                        nsw: _,
                        nuw: _,
                        args,
                    } => {
                        let x = genvalue(&func.data, &id_to_genvalue, args[0]);
                        let y = genvalue(&func.data, &id_to_genvalue, args[1]);
                        match inst.opcode {
                            Opcode::Add => id_to_genvalue.insert(inst_id, add(x, y).unwrap()),
                            Opcode::Sub => id_to_genvalue.insert(inst_id, sub(x, y).unwrap()),
                            _ => todo!(),
                        };
                    }
                    Operand::ICmp { ty: _, args, cond } => {
                        let x = genvalue(&func.data, &id_to_genvalue, args[0]);
                        let y = genvalue(&func.data, &id_to_genvalue, args[1]);
                        let res = match cond {
                            ICmpCond::Slt => slt(x, y).unwrap(),
                            _ => todo!(),
                        };
                        id_to_genvalue.insert(inst_id, res);
                    }
                    Operand::CondBr { arg, blocks } => {
                        let arg = genvalue(&func.data, &id_to_genvalue, *arg);
                        if matches!(arg, GenericValue::Int1(true)) {
                            block = blocks[0];
                        } else {
                            block = blocks[1];
                        }
                        continue 'outer;
                    }
                    Operand::Br { block: b } => {
                        block = *b;
                        continue 'outer;
                    }
                    Operand::Ret { val, .. } if val.is_none() => return Some(GenericValue::Void),
                    Operand::Ret {
                        ty: _,
                        val: Some(val),
                    } => {
                        let val = genvalue(&func.data, &id_to_genvalue, *val);
                        return Some(val);
                    }
                    _ => todo!("{:?}", inst.opcode),
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

fn genvalue(
    data: &Data,
    id_to_genvalue: &FxHashMap<InstructionId, GenericValue>,
    id: ValueId,
) -> GenericValue {
    match data.value_ref(id) {
        Value::Instruction(id) => id_to_genvalue[id],
        Value::Constant(ConstantData::Int(ConstantInt::Int32(i))) => GenericValue::Int32(*i),
        _ => todo!(),
    }
}

fn add(x: GenericValue, y: GenericValue) -> Option<GenericValue> {
    match (x, y) {
        (GenericValue::Int32(x), GenericValue::Int32(y)) => Some(GenericValue::Int32(x + y)),
        _ => None,
    }
}

fn sub(x: GenericValue, y: GenericValue) -> Option<GenericValue> {
    match (x, y) {
        (GenericValue::Int32(x), GenericValue::Int32(y)) => Some(GenericValue::Int32(x - y)),
        _ => None,
    }
}

fn slt(x: GenericValue, y: GenericValue) -> Option<GenericValue> {
    match (x, y) {
        (GenericValue::Int32(x), GenericValue::Int32(y)) => Some(GenericValue::Int1(x < y)),
        _ => None,
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
