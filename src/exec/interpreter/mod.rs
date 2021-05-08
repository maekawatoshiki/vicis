mod frame;

use super::generic_value::GenericValue;
use crate::ir::{
    function::{
        instruction::{ICmpCond, InstructionId, Opcode, Operand},
        FunctionId,
    },
    module::Module,
    types::{Type, TypeId, Types},
    value::{ConstantData, ConstantInt, Value},
};
use std::alloc;

pub fn run_function(module: &Module, func_id: FunctionId) -> Option<GenericValue> {
    let func = &module.functions()[func_id];

    let mut stack = frame::StackFrame::new(module, func);

    let mut block = func.layout.first_block?;

    'main: loop {
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
                    align,
                } => run_alloca(&mut stack, inst_id, tys, num_elements, *align),
                Operand::Store {
                    tys,
                    args,
                    align: _,
                } => {
                    let ty = tys[0];
                    let src = args[0];
                    let dst = args[1];
                    let dst_addr = stack.get_val(dst).unwrap();
                    match func.data.value_ref(src) {
                        Value::Constant(ConstantData::Int(ConstantInt::Int32(i))) => unsafe {
                            *(dst_addr.to_ptr().unwrap() as *mut i32) = *i;
                        },
                        Value::Instruction(id) if matches!(&*func.types.get(ty), Type::Int(32)) => unsafe {
                            *(dst_addr.to_ptr().unwrap() as *mut i32) =
                                stack.get_inst_val(*id).unwrap().to_i32().unwrap();
                        },
                        e => todo!("{:?}", e),
                    }
                }
                Operand::Load {
                    tys,
                    addr,
                    align: _,
                } => {
                    let ty = tys[0];
                    let addr = stack.get_val(*addr).unwrap();
                    match &*func.types.get(ty) {
                        Type::Int(32) => stack.add_inst_val(
                            inst_id,
                            GenericValue::Int32(unsafe { *(addr.to_ptr().unwrap() as *const i32) }),
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
                    let x = stack.get_val(args[0]).unwrap();
                    let y = stack.get_val(args[1]).unwrap();
                    match inst.opcode {
                        Opcode::Add => stack.add_inst_val(inst_id, add(x, y).unwrap()),
                        Opcode::Sub => stack.add_inst_val(inst_id, sub(x, y).unwrap()),
                        _ => todo!(),
                    };
                }
                Operand::ICmp { ty: _, args, cond } => {
                    let x = stack.get_val(args[0]).unwrap();
                    let y = stack.get_val(args[1]).unwrap();
                    let res = match cond {
                        ICmpCond::Slt => slt(x, y).unwrap(),
                        _ => todo!(),
                    };
                    stack.add_inst_val(inst_id, res);
                }
                Operand::CondBr { arg, blocks } => {
                    let arg = stack.get_val(*arg).unwrap();
                    if matches!(arg, GenericValue::Int1(true)) {
                        block = blocks[0];
                    } else {
                        block = blocks[1];
                    }
                    continue 'main;
                }
                Operand::Br { block: b } => {
                    block = *b;
                    continue 'main;
                }
                Operand::Ret { val, .. } if val.is_none() => return Some(GenericValue::Void),
                Operand::Ret {
                    ty: _,
                    val: Some(val),
                } => {
                    let val = stack.get_val(*val).unwrap();
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

// Instructions

fn run_alloca(
    stack: &mut frame::StackFrame,
    id: InstructionId,
    tys: &[TypeId],
    num_elements: &ConstantData,
    align: u32,
) {
    let alloc_ty = tys[0];
    let alloc_sz = stack.func.types.size_of(alloc_ty) * num_elements.as_int().cast_to_usize();
    let alloc_align = if align > 0 { align } else { 8 } as usize;
    let ptr = unsafe {
        alloc::alloc(alloc::Layout::from_size_align(alloc_sz, alloc_align).expect("layout err"))
    };
    stack.add_inst_val(id, GenericValue::Ptr(ptr));
}

// Utils

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
