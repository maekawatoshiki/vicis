mod frame;

extern crate libffi;
extern crate libloading;

use super::generic_value::GenericValue;
use frame::StackFrame;
use rustc_hash::FxHashMap;
use std::{alloc, ffi, os::raw::c_void, ptr};
use vicis_core::ir::{
    function::{
        basic_block::BasicBlockId,
        instruction::{
            Alloca, Br, Call, Cast, CondBr, GetElementPtr, ICmp, ICmpCond, InstructionId,
            IntBinary, Invoke, Load, Opcode, Operand, Phi, Ret, Store,
        },
        Function, FunctionId,
    },
    module::{linkage::Linkage, name::Name, Module},
    types::{self, Type, Typed, Types},
    value::{ConstantArray, ConstantStruct, ConstantValue, ValueId},
};

/// An execution context for interpreters.
pub struct Context<'a> {
    pub module: &'a Module,
    globals: FxHashMap<Name, GenericValue>,
    libs: Vec<libloading::Library>,
}

/// A builder for `Context`.
pub struct ContextBuilder<'a> {
    module: &'a Module,
    globals: FxHashMap<Name, GenericValue>,
    libs: Vec<Result<libloading::Library, libloading::Error>>,
}

pub fn run_function(
    ctx: &Context,
    func_id: FunctionId,
    args: Vec<GenericValue>,
) -> Option<GenericValue> {
    let func = &ctx.module.functions()[func_id];

    if func.is_prototype() {
        return Some(call_external_func(ctx, func, &args));
    }

    let mut frame = StackFrame::new(ctx, func, args);
    let mut block = func.layout.first_block?;
    let mut last_block = block; // TODO: We need a more elegant way.

    'main: loop {
        for (inst_id, inst) in func
            .layout
            .inst_iter(block)
            .into_iter()
            .map(|id| (id, func.data.inst_ref(id)))
        {
            match &inst.operand {
                Operand::Alloca(Alloca {
                    tys,
                    num_elements,
                    align,
                }) => run_alloca(&mut frame, inst_id, tys, num_elements, *align),
                Operand::Phi(Phi {
                    ty: _,
                    args,
                    blocks,
                }) => run_phi(&mut frame, last_block, inst_id, args, blocks),
                Operand::Store(Store { tys, args, align }) => {
                    run_store(&mut frame, tys, args, *align)
                }
                Operand::Load(Load { tys, addr, align }) => {
                    run_load(&mut frame, inst_id, tys, *addr, *align)
                }
                Operand::IntBinary(IntBinary {
                    ty: _,
                    nsw: _,
                    nuw: _,
                    exact: _,
                    args,
                }) => run_int_binary(&mut frame, inst_id, inst.opcode, args),
                Operand::ICmp(ICmp { ty: _, args, cond }) => {
                    run_icmp(&mut frame, inst_id, args, *cond)
                }
                Operand::Cast(Cast { tys, arg }) => {
                    run_cast(&mut frame, inst_id, inst.opcode, tys, *arg)
                }
                Operand::GetElementPtr(GetElementPtr {
                    inbounds: _,
                    tys,
                    args,
                }) => run_gep(&mut frame, inst_id, tys, args),
                Operand::Call(Call { tys, args, .. }) => run_call(&mut frame, inst_id, tys, args),
                Operand::Invoke(Invoke {
                    tys, args, blocks, ..
                }) => {
                    run_call(&mut frame, inst_id, tys, args);
                    // TODO: Add support for exception label.
                    last_block = block;
                    block = blocks[0];
                    continue 'main;
                }
                Operand::CondBr(CondBr { arg, blocks }) => {
                    let arg = frame.get_val(*arg).unwrap();
                    last_block = block;
                    block = blocks[if matches!(arg, GenericValue::Int1(true)) {
                        0
                    } else {
                        1
                    }];
                    continue 'main;
                }
                Operand::Br(Br { block: b }) => {
                    last_block = block;
                    block = *b;
                    continue 'main;
                }
                Operand::Ret(Ret { val, .. }) if val.is_none() => return Some(GenericValue::Void),
                Operand::Ret(Ret {
                    ty: _,
                    val: Some(val),
                }) => {
                    let val = frame.get_val(*val).unwrap();
                    return Some(val);
                }
                _ => todo!("{:?}", inst.opcode),
            }
        }

        if let Some(next) = func.layout.next_block_of(block) {
            last_block = block;
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
    tys: &[Type],
    num_elements: &ConstantValue,
    align: u32,
) {
    let alloc_ty = tys[0];
    let alloc_sz = frame
        .ctx
        .module
        .target()
        .datalayout
        .get_size_of(&frame.func.types, alloc_ty)
        * num_elements.as_int().unwrap().cast_to_usize();
    let alloc_align = if align > 0 { align } else { 8 } as usize;
    let ptr = unsafe {
        alloc::alloc_zeroed(
            alloc::Layout::from_size_align(alloc_sz, alloc_align).expect("layout err"),
        )
    };
    frame.set_inst_val(id, GenericValue::Ptr(ptr));
}

fn run_phi(
    frame: &mut StackFrame,
    last_block: BasicBlockId,
    id: InstructionId,
    args: &[ValueId],
    blocks: &[BasicBlockId],
) {
    let idx = blocks
        .iter()
        .position(|&block| block == last_block)
        .unwrap(); // TODO: It may be slow to iterate over blocks.
    let val = frame.get_val(args[idx]).unwrap();
    frame.set_inst_val(id, val);
}

fn run_store(frame: &mut StackFrame, _tys: &[Type], args: &[ValueId], _align: u32) {
    let src = args[0];
    let dst = args[1];
    let dst = frame.get_val(dst).unwrap().to_ptr().unwrap();
    let src = frame.get_val(src).unwrap();
    match src {
        GenericValue::Int1(i) => unsafe { *(dst as *mut i8) = i as i8 },
        GenericValue::Int8(i) => unsafe { *(dst as *mut i8) = i },
        GenericValue::Int32(i) => unsafe { *(dst as *mut i32) = i },
        GenericValue::Int64(i) => unsafe { *(dst as *mut i64) = i },
        GenericValue::Ptr(p) => unsafe { *(dst as *mut *mut u8) = p },
        t => todo!("{:?}", t),
    }
}

fn run_load(frame: &mut StackFrame, id: InstructionId, tys: &[Type], addr: ValueId, _align: u32) {
    let ty = tys[0];
    let addr = frame.get_val(addr).unwrap().to_ptr().unwrap();
    let val = match ty {
        types::I8 => GenericValue::Int8(unsafe { *(addr as *const i8) }),
        types::I16 => GenericValue::Int16(unsafe { *(addr as *const i16) }),
        types::I32 => GenericValue::Int32(unsafe { *(addr as *const i32) }),
        types::I64 => GenericValue::Int64(unsafe { *(addr as *const i64) }),
        _ if ty.is_pointer(&frame.func.types) => {
            GenericValue::Ptr(unsafe { *(addr as *const *mut u8) })
        }
        ty => todo!("Unsupported load type: {}", frame.func.types.to_string(ty)),
    };
    frame.set_inst_val(id, val);
}

fn run_int_binary(frame: &mut StackFrame, id: InstructionId, opcode: Opcode, args: &[ValueId]) {
    let x = frame.get_val(args[0]).unwrap();
    let y = frame.get_val(args[1]).unwrap();
    match opcode {
        Opcode::Add => frame.set_inst_val(id, add(x, y).unwrap()),
        Opcode::Sub => frame.set_inst_val(id, sub(x, y).unwrap()),
        Opcode::Mul => frame.set_inst_val(id, mul(x, y).unwrap()),
        Opcode::SDiv => frame.set_inst_val(id, sdiv(x, y).unwrap()),
        Opcode::SRem => frame.set_inst_val(id, srem(x, y).unwrap()),
        Opcode::Shl => frame.set_inst_val(id, shl(x, y).unwrap()),
        Opcode::AShr => frame.set_inst_val(id, ashr(x, y).unwrap()),
        Opcode::And => frame.set_inst_val(id, and(x, y).unwrap()),
        op => todo!("{:?}", op),
    };
}

fn run_icmp(frame: &mut StackFrame, id: InstructionId, args: &[ValueId], cond: ICmpCond) {
    let x = frame.get_val(args[0]).unwrap();
    let y = frame.get_val(args[1]).unwrap();
    let res = match cond {
        ICmpCond::Eq => eq(x, y).unwrap(),
        ICmpCond::Ne => ne(x, y).unwrap(),
        ICmpCond::Ugt => ugt(x, y).unwrap(),
        ICmpCond::Uge => uge(x, y).unwrap(),
        ICmpCond::Ult => ult(x, y).unwrap(),
        ICmpCond::Ule => ule(x, y).unwrap(),
        ICmpCond::Slt => slt(x, y).unwrap(),
        ICmpCond::Sle => sle(x, y).unwrap(),
        ICmpCond::Sgt => sgt(x, y).unwrap(),
        ICmpCond::Sge => sge(x, y).unwrap(),
    };
    frame.set_inst_val(id, res);
}

fn run_cast(frame: &mut StackFrame, id: InstructionId, opcode: Opcode, tys: &[Type], arg: ValueId) {
    let _from = tys[0];
    let to = tys[1];
    let arg = frame.get_val(arg).unwrap();
    let val = match opcode {
        Opcode::Sext => {
            let arg = arg.sext_to_i64().unwrap();
            match to {
                types::I32 => GenericValue::Int32(arg as i32),
                types::I64 => GenericValue::Int64(arg),
                _ => todo!(),
            }
        }
        Opcode::Trunc => {
            let arg = arg.sext_to_i64().unwrap();
            match to {
                types::I1 => GenericValue::Int1(arg != 0),
                types::I8 => GenericValue::Int8(arg as i8),
                types::I32 => GenericValue::Int32(arg as i32),
                types::I64 => GenericValue::Int64(arg),
                _ => todo!(),
            }
        }
        Opcode::Bitcast => {
            assert!(matches!(arg, GenericValue::Ptr(_)));
            assert!(to.is_pointer(&frame.func.types));
            arg
        }
        Opcode::Zext => {
            let arg = arg.zext_to_u64().unwrap();
            match to {
                types::I32 => GenericValue::Int32(arg as i32),
                types::I64 => GenericValue::Int64(arg as i64),
                _ => todo!(),
            }
        }
        t => todo!("cast {:?}", t),
    };
    frame.set_inst_val(id, val)
}

fn run_gep(frame: &mut StackFrame, id: InstructionId, tys: &[Type], args: &[ValueId]) {
    let arg = frame.get_val(args[0]).unwrap().to_ptr().unwrap();
    let mut total = 0;
    let mut cur_ty = tys[1];
    let dl = &frame.ctx.module.target().datalayout;
    for &idx in &args[1..] {
        let idx = match frame.get_val(idx).unwrap() {
            GenericValue::Int32(idx) => idx as isize,
            GenericValue::Int64(idx) => idx as isize,
            _ => panic!(),
        };
        if cur_ty.is_struct(&frame.func.types) {
            let sl = dl
                .new_struct_layout_for(&frame.func.types, cur_ty)
                .expect("cur_ty must be struct");
            let offset = sl.get_elem_offset(idx as usize).unwrap() as isize;
            let inner = frame
                .func
                .types
                .base()
                .element_at(cur_ty, idx as usize)
                .unwrap();
            total += offset;
            cur_ty = inner;
        } else {
            let inner = frame.func.types.get_element(cur_ty).unwrap();
            total += dl.get_size_of(&frame.func.types, inner) as isize * idx;
            cur_ty = inner;
        }
    }
    frame.set_inst_val(id, GenericValue::Ptr(unsafe { arg.offset(total) }));
}

fn run_call(frame: &mut StackFrame, id: InstructionId, _tys: &[Type], args: &[ValueId]) {
    let callee = frame.get_val(args[0]).unwrap();
    let args: Vec<GenericValue> = args[1..]
        .iter()
        .map(|&a| frame.get_val(a).unwrap())
        .collect();
    let func_id = callee.to_id::<FunctionId>().unwrap();
    if let Some(ret) = run_function(frame.ctx, *func_id, args) {
        match ret {
            GenericValue::Void => {}
            v => frame.set_inst_val(id, v),
        }
    }
}

// Utils

fn add(x: GenericValue, y: GenericValue) -> Option<GenericValue> {
    match (x, y) {
        (GenericValue::Int8(x), GenericValue::Int8(y)) => Some(GenericValue::Int8(x + y)),
        (GenericValue::Int32(x), GenericValue::Int32(y)) => Some(GenericValue::Int32(x + y)),
        (GenericValue::Int64(x), GenericValue::Int64(y)) => Some(GenericValue::Int64(x + y)),
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

fn shl(x: GenericValue, y: GenericValue) -> Option<GenericValue> {
    match (x, y) {
        (GenericValue::Int32(x), GenericValue::Int32(y)) => {
            Some(GenericValue::Int32(x.rotate_left(y as u32)))
        }
        (GenericValue::Int64(x), GenericValue::Int64(y)) => {
            Some(GenericValue::Int64(x.rotate_left(y as u32)))
        }
        _ => None,
    }
}

fn ashr(x: GenericValue, y: GenericValue) -> Option<GenericValue> {
    match (x, y) {
        (GenericValue::Int32(x), GenericValue::Int32(y)) => Some(GenericValue::Int32(x >> y)),
        (GenericValue::Int64(x), GenericValue::Int64(y)) => Some(GenericValue::Int64(x >> y)),
        _ => None,
    }
}

fn and(x: GenericValue, y: GenericValue) -> Option<GenericValue> {
    match (x, y) {
        (GenericValue::Int32(x), GenericValue::Int32(y)) => Some(GenericValue::Int32(x & y)),
        (GenericValue::Int64(x), GenericValue::Int64(y)) => Some(GenericValue::Int64(x & y)),
        _ => None,
    }
}

fn eq(x: GenericValue, y: GenericValue) -> Option<GenericValue> {
    match (x, y) {
        (GenericValue::Int1(x), GenericValue::Int1(y)) => Some(GenericValue::Int1(x == y)),
        (GenericValue::Int32(x), GenericValue::Int32(y)) => Some(GenericValue::Int1(x == y)),
        (GenericValue::Ptr(x), GenericValue::Ptr(y)) => Some(GenericValue::Int1(x == y)),
        _ => None,
    }
}

fn ne(x: GenericValue, y: GenericValue) -> Option<GenericValue> {
    match (x, y) {
        (GenericValue::Int1(x), GenericValue::Int1(y)) => Some(GenericValue::Int1(x != y)),
        (GenericValue::Int32(x), GenericValue::Int32(y)) => Some(GenericValue::Int1(x != y)),
        _ => None,
    }
}

fn ult(x: GenericValue, y: GenericValue) -> Option<GenericValue> {
    match (x, y) {
        (GenericValue::Int32(x), GenericValue::Int32(y)) => Some(GenericValue::Int1(x < y)),
        _ => None,
    }
}

fn ule(x: GenericValue, y: GenericValue) -> Option<GenericValue> {
    match (x, y) {
        (GenericValue::Int32(x), GenericValue::Int32(y)) => Some(GenericValue::Int1(x <= y)),
        _ => None,
    }
}

fn ugt(x: GenericValue, y: GenericValue) -> Option<GenericValue> {
    match (x, y) {
        (GenericValue::Int32(x), GenericValue::Int32(y)) => Some(GenericValue::Int1(x > y)),
        _ => None,
    }
}

fn uge(x: GenericValue, y: GenericValue) -> Option<GenericValue> {
    match (x, y) {
        (GenericValue::Int32(x), GenericValue::Int32(y)) => Some(GenericValue::Int1(x >= y)),
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

fn sgt(x: GenericValue, y: GenericValue) -> Option<GenericValue> {
    match (x, y) {
        (GenericValue::Int32(x), GenericValue::Int32(y)) => Some(GenericValue::Int1(x > y)),
        _ => None,
    }
}

fn sge(x: GenericValue, y: GenericValue) -> Option<GenericValue> {
    match (x, y) {
        (GenericValue::Int32(x), GenericValue::Int32(y)) => Some(GenericValue::Int1(x >= y)),
        _ => None,
    }
}

// Context

impl<'a> ContextBuilder<'a> {
    pub fn new(module: &'a Module) -> Self {
        ContextBuilder {
            module,
            globals: FxHashMap::default(),
            libs: vec![],
        }
    }

    pub fn with_lib<T: AsRef<ffi::OsStr>>(mut self, lib: T) -> Self {
        self.libs.push(unsafe { libloading::Library::new(lib) });
        self
    }

    pub fn with_libs<T: AsRef<ffi::OsStr>>(mut self, libs: Vec<T>) -> Self {
        for lib in libs {
            self.libs.push(unsafe { libloading::Library::new(lib) });
        }
        self
    }

    pub fn build(self) -> Result<Context<'a>, libloading::Error> {
        let mut ctx = Context {
            module: self.module,
            globals: self.globals,
            libs: self.libs.into_iter().collect::<Result<_, _>>()?,
        };

        let mut ctor = None;
        let dl = &ctx.module.target().datalayout;

        for (name, gv) in ctx.module.global_variables() {
            let sz = dl.get_size_of(&ctx.module.types, gv.ty);
            let align = if gv.align > 0 { gv.align } else { 8 } as usize;
            let special = matches!(name, Name::Name(ref n) if n == "__dso_handle"); // TODO: Better find another way to treat this.
            if matches!(
                gv.linkage,
                Some(Linkage::External) | Some(Linkage::ExternalWeak)
            ) && !special
            {
                let p = *ctx
                    .lookup::<*mut u8>(name.as_string().as_str())
                    .expect("external not found");
                ctx.globals.insert(name.clone(), GenericValue::Ptr(p));
                continue;
            }
            let ptr = unsafe {
                alloc::alloc_zeroed(alloc::Layout::from_size_align(sz, align).expect("layout err"))
            };
            if let Some(init) = &gv.init {
                match init {
                    // Handle 'llvm.global_ctors'
                    ConstantValue::Array(ConstantArray {
                        is_string: false,
                        elems,
                        ..
                    }) if matches!(gv.name, Name::Name(ref name) if name == "llvm.global_ctors") => {
                        {
                            assert!(gv.ty.is_array(&ctx.module.types));
                            let strukt = ctx.module.types.get_element(gv.ty).unwrap();
                            let t0 = ctx.module.types.base().element_at(strukt, 0).unwrap();
                            let t1 = ctx.module.types.base().element_at(strukt, 1).unwrap();
                            let t2 = ctx.module.types.base().element_at(strukt, 2).unwrap();
                            assert!(t0.is_i32()); // i32
                            assert!(t1.is_pointer(&ctx.module.types)); // void ()*
                            assert!(t2.is_pointer(&ctx.module.types)); // i8*
                            assert!(elems.len() == 1);
                        }
                        if let ConstantValue::Struct(ConstantStruct { elems, .. }) = &elems[0] {
                            if let ConstantValue::GlobalRef(name, _) = &elems[1] {
                                ctor = ctx.module.find_function_by_name(name.as_string());
                            }
                        } else {
                            todo!()
                        }
                    }
                    _ => init_memory(&ctx, init, ptr as *mut i8),
                }
            }
            ctx.globals.insert(name.clone(), GenericValue::Ptr(ptr));
        }

        if let Some(ctor) = ctor {
            run_function(&ctx, ctor, vec![]);
        }

        Ok(ctx)
    }
}

fn init_memory(ctx: &Context, val: &ConstantValue, ptr: *mut i8) {
    let dl = &ctx.module.target().datalayout;
    match val {
        ConstantValue::Array(ConstantArray { elems, elem_ty, .. }) => {
            let sz = dl.get_size_of(&ctx.module.types, *elem_ty);
            for (i, e) in elems.iter().enumerate() {
                init_memory(ctx, e, unsafe { ptr.add(sz * i) as *mut i8 });
            }
        }
        ConstantValue::Struct(ConstantStruct { ty, elems, .. }) => {
            let layout = dl.new_struct_layout_for(&ctx.module.types, *ty).unwrap();
            for (i, e) in elems.iter().enumerate() {
                let offset = layout.get_elem_offset(i).unwrap();
                init_memory(ctx, e, unsafe { ptr.add(offset) as *mut i8 });
            }
        }
        ConstantValue::Undef(_) | ConstantValue::Null(_) | ConstantValue::AggregateZero(_) => {
            // Already zeroed.
            // unsafe { ptr::write_bytes(ptr, 0, sz) };
        }
        ConstantValue::Int(i) => unsafe {
            ptr::copy_nonoverlapping(i.as_ptr(), ptr, dl.get_size_of(&ctx.module.types, i.ty()))
        },
        ConstantValue::GlobalRef(_, _) | ConstantValue::Expr(_) => todo!(),
    }
}

impl<'a> Context<'a> {
    fn lookup<T>(&self, name: &str) -> Option<libloading::Symbol<T>> {
        self.libs
            .iter()
            .find_map(|lib| unsafe { lib.get(name.as_bytes()) }.ok())
    }
}

fn ffitype(ty: Type, types: &Types) -> libffi::low::ffi_type {
    match ty {
        types::VOID => unsafe { libffi::low::types::void },
        types::I32 => unsafe { libffi::low::types::sint32 },
        types::I64 => unsafe { libffi::low::types::sint64 },
        ty if ty.is_pointer(types) => unsafe { libffi::low::types::pointer },
        e => panic!("{:?}", e),
    }
}

fn call_external_func(ctx: &Context, func: &Function, args: &[GenericValue]) -> GenericValue {
    if let Some(ret) = call_intrinsic_func(ctx, func, args) {
        return ret;
    }

    #[cfg(debug_assertions)]
    log::debug!("external enter: {}", func.name);

    let mut args_ty = Vec::with_capacity(args.len());
    let mut new_args = Vec::with_capacity(args.len());
    let mut tmps = vec![]; // Used to store temporary values for libffi invoke.
    let mut args: Vec<GenericValue> = args.to_vec();

    for arg in &mut args {
        match arg {
            GenericValue::Int1(ref mut i) => {
                args_ty.push(unsafe { &mut libffi::low::types::uint8 as *mut _ });
                new_args.push(i as *mut _ as *mut c_void)
            }
            GenericValue::Int8(ref mut i) => {
                args_ty.push(unsafe { &mut libffi::low::types::sint8 as *mut _ });
                new_args.push(i as *mut _ as *mut c_void)
            }
            GenericValue::Int32(ref mut i) => {
                args_ty.push(unsafe { &mut libffi::low::types::sint32 as *mut _ });
                new_args.push(i as *mut _ as *mut c_void)
            }
            GenericValue::Int64(ref mut i) => {
                args_ty.push(unsafe { &mut libffi::low::types::sint64 as *mut _ });
                new_args.push(i as *mut _ as *mut c_void)
            }
            GenericValue::Ptr(ref mut p) => {
                args_ty.push(unsafe { &mut libffi::low::types::pointer as *mut _ });
                new_args.push(&mut *p as *mut _ as *mut c_void);
            }
            GenericValue::Id(_) => {
                // If `arg` is (an id for) an external function, get its address.
                if let Some(id) = arg.to_id::<FunctionId>() {
                    let f = &ctx.module.functions()[*id];
                    let sym = ctx
                        .lookup::<*const u8>(&f.name)
                        .map_or(dummy_func as *const u8, |s| *s);
                    tmps.push(sym);
                    args_ty.push(unsafe { &mut libffi::low::types::pointer as *mut _ });
                    new_args.push(&mut *tmps.last_mut().unwrap() as *mut _ as *mut c_void);
                    continue;
                }
                todo!();
                // args_ty.push(unsafe { &mut libffi::low::types::pointer as *mut _ });
                // new_args.push(&mut 0 as *mut _ as *mut c_void);
            }
            e => todo!("{:?}", e),
        }
    }

    let mut ret_ty = ffitype(func.result_ty, &func.types);
    let mut cif: libffi::low::ffi_cif = Default::default();
    let prms_len = func.params.len();
    let func1 = ctx.lookup::<unsafe extern "C" fn()>(func.name()).unwrap();
    let func1 = libffi::low::CodePtr(unsafe { func1.into_raw() }.into_raw());

    unsafe {
        libffi::low::prep_cif_var(
            &mut cif,
            libffi::low::ffi_abi_FFI_DEFAULT_ABI,
            prms_len,
            args_ty.len(),
            &mut ret_ty,
            args_ty.as_mut_ptr(),
        )
    }
    .unwrap();

    let ret = match func.result_ty {
        types::VOID => {
            unsafe { libffi::low::call::<c_void>(&mut cif, func1, new_args.as_mut_ptr()) };
            GenericValue::Void
        }
        types::I32 => {
            let r: i32 = unsafe { libffi::low::call(&mut cif, func1, new_args.as_mut_ptr()) };
            GenericValue::Int32(r)
        }
        types::I64 => {
            let r: i64 = unsafe { libffi::low::call(&mut cif, func1, new_args.as_mut_ptr()) };
            GenericValue::Int64(r)
        }
        ty if ty.is_pointer(&func.types) => {
            let r: *mut u8 = unsafe { libffi::low::call(&mut cif, func1, new_args.as_mut_ptr()) };
            GenericValue::Ptr(r)
        }
        _ => panic!(),
    };

    #[cfg(debug_assertions)]
    log::debug!("external exit: {}", func.name);

    ret
}

fn call_intrinsic_func(
    _ctx: &Context,
    func: &Function,
    args: &[GenericValue],
) -> Option<GenericValue> {
    fn llvm_memcpy_p0i8_p0i8_i64(args: &[GenericValue]) -> GenericValue {
        let dst = args[0].to_ptr().unwrap();
        let src = args[1].to_ptr().unwrap();
        let len = args[2].to_i64().unwrap();
        let _is_volatile = args[3].to_i1().unwrap();
        unsafe { ptr::copy_nonoverlapping(src, dst, len as usize) }
        GenericValue::Void
    }

    fn llvm_memset_p0i8_i64(args: &[GenericValue]) -> GenericValue {
        let dst = args[0].to_ptr().unwrap();
        let val = args[1].to_i8().unwrap();
        let len = args[2].to_i64().unwrap();
        let _is_volatile = args[3].to_i1().unwrap();
        unsafe { ptr::write_bytes(dst, val as u8, len as usize) }
        GenericValue::Void
    }

    let funcs: FxHashMap<&'static str, fn(&[GenericValue]) -> GenericValue> = vec![
        (
            "llvm.memcpy.p0i8.p0i8.i64",
            llvm_memcpy_p0i8_p0i8_i64 as fn(&[GenericValue]) -> GenericValue,
        ),
        ("llvm.memset.p0i8.i64", llvm_memset_p0i8_i64),
    ]
    .into_iter()
    .collect();

    if let Some(func) = funcs.get(func.name().as_str()) {
        return Some(func(args));
    }

    None
}

extern "C" fn dummy_func() {
    log::info!("dummy_func: passing pointers of interpreter functions to external functions is not suppported.");
}
