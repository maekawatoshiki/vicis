mod frame;

extern crate libffi;
extern crate libloading;

use super::generic_value::GenericValue;
use crate::ir::{
    function::{
        instruction::{ICmpCond, InstructionId, Opcode, Operand},
        Function, FunctionId,
    },
    module::{name::Name, Module},
    types::{ArrayType, Type, TypeId, Types},
    value::{ConstantArray, ConstantData, ValueId},
};
use frame::StackFrame;
use rustc_hash::FxHashMap;
use std::{alloc, ffi, os::raw::c_void, ptr};

pub struct Context<'a> {
    pub module: &'a Module,
    globals: FxHashMap<Name, GenericValue>,
    libs: Vec<libloading::Library>,
}

pub fn run_function(
    ctx: &Context,
    func_id: FunctionId,
    args: Vec<GenericValue>,
) -> Option<GenericValue> {
    let func = &ctx.module.functions()[func_id];

    if func.is_prototype {
        return Some(call_external_func(ctx, func, &args));
    }

    let mut frame = StackFrame::new(ctx, func, args);
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
                } => run_alloca(&mut frame, inst_id, tys, num_elements, *align),
                Operand::Store { tys, args, align } => run_store(&mut frame, tys, args, *align),
                Operand::Load { tys, addr, align } => {
                    run_load(&mut frame, inst_id, tys, *addr, *align)
                }
                Operand::IntBinary {
                    ty: _,
                    nsw: _,
                    nuw: _,
                    exact: _,
                    args,
                } => run_int_binary(&mut frame, inst_id, inst.opcode, args),
                Operand::ICmp { ty: _, args, cond } => run_icmp(&mut frame, inst_id, args, *cond),
                Operand::Cast { tys, arg } => run_cast(&mut frame, inst_id, inst.opcode, tys, *arg),
                Operand::GetElementPtr {
                    inbounds: _,
                    tys,
                    args,
                } => run_gep(&mut frame, inst_id, tys, args),
                Operand::Call { tys, args, .. } => run_call(&mut frame, inst_id, tys, args),
                Operand::CondBr { arg, blocks } => {
                    let arg = frame.get_val(*arg).unwrap();
                    block = blocks[if matches!(arg, GenericValue::Int1(true)) {
                        0
                    } else {
                        1
                    }];
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
                    let val = frame.get_val(*val).unwrap();
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
    frame: &mut StackFrame,
    id: InstructionId,
    tys: &[TypeId],
    num_elements: &ConstantData,
    align: u32,
) {
    let alloc_ty = tys[0];
    let alloc_sz = frame.func.types.size_of(alloc_ty) * num_elements.as_int().cast_to_usize();
    let alloc_align = if align > 0 { align } else { 8 } as usize;
    let ptr = unsafe {
        alloc::alloc(alloc::Layout::from_size_align(alloc_sz, alloc_align).expect("layout err"))
    };
    frame.add_inst_val(id, GenericValue::Ptr(ptr));
}

fn run_store(frame: &mut StackFrame, _tys: &[TypeId], args: &[ValueId], _align: u32) {
    let src = args[0];
    let dst = args[1];
    let dst = frame.get_val(dst).unwrap();
    let src = frame.get_val(src).unwrap();
    match src {
        GenericValue::Int32(i) => unsafe {
            *(dst.to_ptr().unwrap() as *mut i32) = i;
        },
        GenericValue::Ptr(p) => unsafe {
            *(dst.to_ptr().unwrap() as *mut *mut u8) = p;
        },
        t => todo!("{:?}", t),
    }
}

fn run_load(frame: &mut StackFrame, id: InstructionId, tys: &[TypeId], addr: ValueId, _align: u32) {
    let ty = tys[0];
    let addr = frame.get_val(addr).unwrap();
    match &*frame.func.types.get(ty) {
        Type::Int(8) => frame.add_inst_val(
            id,
            GenericValue::Int8(unsafe { *(addr.to_ptr().unwrap() as *const i8) }),
        ),
        Type::Int(32) => frame.add_inst_val(
            id,
            GenericValue::Int32(unsafe { *(addr.to_ptr().unwrap() as *const i32) }),
        ),
        Type::Pointer(_) => frame.add_inst_val(
            id,
            GenericValue::Ptr(unsafe { *(addr.to_ptr().unwrap() as *const *mut u8) }),
        ),
        _ => todo!(),
    };
}

fn run_int_binary(frame: &mut StackFrame, id: InstructionId, opcode: Opcode, args: &[ValueId]) {
    let x = frame.get_val(args[0]).unwrap();
    let y = frame.get_val(args[1]).unwrap();
    match opcode {
        Opcode::Add => frame.add_inst_val(id, add(x, y).unwrap()),
        Opcode::Sub => frame.add_inst_val(id, sub(x, y).unwrap()),
        Opcode::Mul => frame.add_inst_val(id, mul(x, y).unwrap()),
        Opcode::SDiv => frame.add_inst_val(id, sdiv(x, y).unwrap()),
        Opcode::SRem => frame.add_inst_val(id, srem(x, y).unwrap()),
        _ => todo!(),
    };
}

fn run_icmp(frame: &mut StackFrame, id: InstructionId, args: &[ValueId], cond: ICmpCond) {
    let x = frame.get_val(args[0]).unwrap();
    let y = frame.get_val(args[1]).unwrap();
    let res = match cond {
        ICmpCond::Slt => slt(x, y).unwrap(),
        ICmpCond::Sle => sle(x, y).unwrap(),
        _ => todo!(),
    };
    frame.add_inst_val(id, res);
}

fn run_cast(
    frame: &mut StackFrame,
    id: InstructionId,
    opcode: Opcode,
    tys: &[TypeId],
    arg: ValueId,
) {
    let _from = tys[0];
    let to = tys[1];
    let arg = frame.get_val(arg).unwrap();
    let val = match opcode {
        Opcode::Sext => {
            let arg = arg.sext_to_i64().unwrap();
            match &*frame.func.types.get(to) {
                Type::Int(32) => GenericValue::Int32(arg as i32),
                Type::Int(64) => GenericValue::Int64(arg),
                _ => todo!(),
            }
        }
        t => todo!("cast {:?}", t),
    };
    frame.add_inst_val(id, val)
}

fn run_gep(frame: &mut StackFrame, id: InstructionId, tys: &[TypeId], args: &[ValueId]) {
    let arg = frame.get_val(args[0]).unwrap().to_ptr().unwrap();
    let mut total = 0;
    let mut cur_ty = tys[1];
    for &idx in &args[1..] {
        if matches!(&*frame.func.types.get(cur_ty), Type::Struct(_)) {
        } else {
            let inner = frame.func.types.get_element(cur_ty).unwrap();
            let idx = match frame.get_val(idx).unwrap() {
                GenericValue::Int32(idx) => idx as usize,
                GenericValue::Int64(idx) => idx as usize,
                _ => panic!(),
            };
            total += frame.func.types.size_of(inner) * idx;
            cur_ty = inner;
        }
    }
    frame.add_inst_val(id, GenericValue::Ptr(unsafe { arg.add(total) }));
}

fn run_call(frame: &mut StackFrame, id: InstructionId, _tys: &[TypeId], args: &[ValueId]) {
    let callee = frame.get_val(args[0]).unwrap();
    let args: Vec<GenericValue> = args[1..]
        .iter()
        .map(|&a| frame.get_val(a).unwrap())
        .collect();
    let func_id = callee.to_id::<FunctionId>().unwrap();
    if let Some(ret) = run_function(frame.ctx, *func_id, args) {
        match ret {
            GenericValue::Void => {}
            v => frame.add_inst_val(id, v),
        }
    }
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

fn mul(x: GenericValue, y: GenericValue) -> Option<GenericValue> {
    match (x, y) {
        (GenericValue::Int32(x), GenericValue::Int32(y)) => Some(GenericValue::Int32(x * y)),
        _ => None,
    }
}

fn sdiv(x: GenericValue, y: GenericValue) -> Option<GenericValue> {
    match (x, y) {
        (GenericValue::Int32(x), GenericValue::Int32(y)) => Some(GenericValue::Int32(x / y)),
        _ => None,
    }
}

fn srem(x: GenericValue, y: GenericValue) -> Option<GenericValue> {
    match (x, y) {
        (GenericValue::Int32(x), GenericValue::Int32(y)) => Some(GenericValue::Int32(x % y)),
        _ => None,
    }
}

fn slt(x: GenericValue, y: GenericValue) -> Option<GenericValue> {
    match (x, y) {
        (GenericValue::Int32(x), GenericValue::Int32(y)) => Some(GenericValue::Int1(x < y)),
        _ => None,
    }
}

fn sle(x: GenericValue, y: GenericValue) -> Option<GenericValue> {
    match (x, y) {
        (GenericValue::Int32(x), GenericValue::Int32(y)) => Some(GenericValue::Int1(x <= y)),
        _ => None,
    }
}

// Context

impl<'a> Context<'a> {
    pub fn new(module: &'a Module) -> Self {
        let mut globals = FxHashMap::default();

        for (name, gv) in &module.global_variables {
            let sz = module.types.size_of(gv.ty);
            let align = if gv.align > 0 { gv.align } else { 8 } as usize;
            let ptr = unsafe {
                alloc::alloc(alloc::Layout::from_size_align(sz, align).expect("layout err"))
            };
            if let Some(init) = &gv.init {
                match init {
                    ConstantData::Array(ConstantArray {
                        is_string: true,
                        elems,
                        ..
                    }) => {
                        let s: Vec<u8> = elems.iter().map(|e| *e.as_int().as_i8() as u8).collect();
                        unsafe { ptr::copy_nonoverlapping(s.as_ptr(), ptr, s.len()) };
                    }
                    _ => todo!(),
                }
            }
            globals.insert(name.clone(), GenericValue::Ptr(ptr));
        }

        Self {
            module,
            globals,
            libs: vec![],
        }
    }

    pub fn with_lib<T: AsRef<ffi::OsStr>>(mut self, lib: T) -> Option<Self> {
        self.libs
            .push(unsafe { libloading::Library::new(lib).ok()? });
        Some(self)
    }

    pub fn with_libs<T: AsRef<ffi::OsStr>>(mut self, libs: Vec<T>) -> Option<Self> {
        for lib in libs {
            self.libs
                .push(unsafe { libloading::Library::new(lib).ok()? });
        }
        Some(self)
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
            Type::Array(ArrayType {
                inner,
                num_elements,
            }) => self.size_of(*inner) * *num_elements as usize,
            Type::Pointer(_) => 8,
            _ => todo!(),
        }
    }
}

fn call_external_func(ctx: &Context, func: &Function, args_: &[GenericValue]) -> GenericValue {
    fn lookup<'a>(
        ctx: &'a Context,
        name: &'a str,
    ) -> Option<libloading::Symbol<'a, unsafe extern "C" fn()>> {
        for lib in &ctx.libs {
            if let Ok(func) = unsafe { lib.get(name.as_bytes()) } {
                return Some(func);
            }
        }
        None
    }

    let func = lookup(ctx, func.name()).unwrap();
    let func = libffi::low::CodePtr(unsafe { func.into_raw() }.into_raw());

    let mut args: Vec<*mut libffi::low::ffi_type> =
        unsafe { vec![&mut libffi::low::types::pointer] };
    let mut cif: libffi::low::ffi_cif = Default::default();

    unsafe {
        libffi::low::prep_cif(
            &mut cif,
            libffi::low::ffi_abi_FFI_DEFAULT_ABI,
            args.len(),
            &mut libffi::low::types::sint32,
            args.as_mut_ptr(),
        )
    }
    .unwrap();

    unsafe {
        libffi::low::call(
            &mut cif,
            func,
            vec![&mut args_[0].to_ptr().unwrap() as *mut _ as *mut c_void].as_mut_ptr(),
        )
    }
}
