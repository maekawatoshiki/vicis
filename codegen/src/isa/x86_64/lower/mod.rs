pub mod load;
pub mod store;

use crate::{
    function::instruction::Instruction as MachInstruction,
    isa::x86_64::{
        instruction::{InstructionData, Opcode, Operand as MO, OperandData},
        register::{RegClass, RegInfo, GR32, GR64, GR8},
        X86_64,
    },
    isa::TargetIsa,
    lower::{Lower as LowerTrait, LoweringContext, LoweringError},
    register::{Reg, RegisterClass, RegisterInfo, VReg},
};
use anyhow::Result;
use load::lower_load;
use store::lower_store;
use vicis_core::ir::{
    function::{
        basic_block::BasicBlockId,
        data::Data as IrData,
        instruction::{
            Alloca, Br, Call, Cast, CondBr, GetElementPtr, ICmp, ICmpCond,
            Instruction as IrInstruction, InstructionId, IntBinary, Load, Opcode as IrOpcode,
            Operand, Phi, Ret, Store,
        },
        Parameter,
    },
    module::name::Name,
    types::{self, CompoundType, FunctionType, Type},
    value::{ConstantExpr, ConstantInt, ConstantValue, Value, ValueId},
};

#[derive(Clone, Copy, Default)]
pub struct Lower {}

impl Lower {
    pub fn new() -> Self {
        Lower::default()
    }
}

impl LowerTrait<X86_64> for Lower {
    fn lower(ctx: &mut LoweringContext<X86_64>, inst: &IrInstruction) -> Result<()> {
        lower(ctx, inst)
    }

    fn copy_args_to_vregs(ctx: &mut LoweringContext<X86_64>, params: &[Parameter]) -> Result<()> {
        let args = RegInfo::arg_reg_list(&ctx.call_conv);
        for (gpr_used, Parameter { name: _, ty, .. }) in params.iter().enumerate() {
            let reg = args[gpr_used].apply(&RegClass::for_type(ctx.types, *ty));
            debug!(reg);
            // Copy reg to new vreg
            assert!(ty.is_integer() || ty.is_pointer(ctx.types));
            let sz = ctx.isa.data_layout().get_size_of(ctx.types, *ty);
            let opcode = match sz {
                1 => Opcode::MOVrr8,
                4 => Opcode::MOVrr32,
                8 => Opcode::MOVrr64,
                _ => todo!(),
            };
            let output = ctx.mach_data.vregs.add_vreg_data(*ty);
            ctx.inst_seq.push(MachInstruction::new(
                InstructionData {
                    opcode,
                    operands: vec![MO::output(output.into()), MO::input(reg.into())],
                },
                ctx.block_map[&ctx.cur_block],
            ));
            ctx.arg_idx_to_vreg.insert(gpr_used, output);
        }
        Ok(())
    }
}

fn lower(ctx: &mut LoweringContext<X86_64>, inst: &IrInstruction) -> Result<()> {
    match inst.operand {
        Operand::Alloca(Alloca {
            ref tys,
            ref num_elements,
            align,
        }) => lower_alloca(ctx, inst.id.unwrap(), tys, num_elements, align),
        Operand::Phi(Phi {
            ty,
            ref args,
            ref blocks,
        }) => lower_phi(ctx, inst.id.unwrap(), ty, args, blocks),
        Operand::Load(Load {
            ref tys,
            addr,
            align,
        }) => lower_load(ctx, inst.id.unwrap(), tys, addr, align),
        Operand::Store(Store {
            ref tys,
            ref args,
            align,
        }) => lower_store(ctx, tys, args, align),
        Operand::IntBinary(IntBinary { ty, ref args, .. }) => {
            lower_bin(ctx, inst.id.unwrap(), inst.opcode, ty, args)
        }
        Operand::Cast(Cast { ref tys, arg }) if inst.opcode == IrOpcode::Sext => {
            lower_sext(ctx, inst.id.unwrap(), tys, arg)
        }
        Operand::Cast(Cast { ref tys, arg }) if inst.opcode == IrOpcode::Bitcast => {
            lower_bitcast(ctx, inst.id.unwrap(), tys, arg)
        }
        Operand::Cast(Cast { ref tys, arg }) if inst.opcode == IrOpcode::Zext => {
            lower_zext(ctx, inst.id.unwrap(), tys, arg)
        }
        Operand::GetElementPtr(ref gep) => lower_gep(ctx, inst.id.unwrap(), gep),
        Operand::Br(Br { block }) => lower_br(ctx, block),
        Operand::CondBr(CondBr { arg, blocks }) => lower_condbr(ctx, arg, blocks),
        Operand::Call(Call {
            ref args, ref tys, ..
        }) => lower_call(ctx, inst.id.unwrap(), tys, args),
        Operand::Ret(Ret { val: None, .. }) => lower_return(ctx, None),
        Operand::Ret(Ret { val: Some(val), ty }) => lower_return(ctx, Some((ty, val))),
        ref e => Err(LoweringError::Todo(format!("Unsupported instruction: {:?}", e)).into()),
    }
}

fn lower_alloca(
    ctx: &mut LoweringContext<X86_64>,
    id: InstructionId,
    tys: &[Type],
    _num_elements: &ConstantValue,
    _align: u32,
) -> Result<()> {
    if let Some(slot_id) = ctx.inst_id_to_slot_id.get(&id) {
        let mem = vec![
            MO::new(OperandData::MemStart),
            MO::new(OperandData::None),
            MO::new(OperandData::Slot(*slot_id)),
            MO::new(OperandData::None),
            MO::input(OperandData::None),
            MO::input(OperandData::None),
            MO::new(OperandData::None),
        ];

        let ty = ctx.types.base_mut().pointer(tys[0]);
        let output = new_empty_inst_output(ctx, ty, id);
        ctx.inst_seq.push(MachInstruction::new(
            InstructionData {
                opcode: Opcode::LEArm64,
                operands: vec![MO::output(output.into())]
                    .into_iter()
                    .chain(mem.into_iter())
                    .collect(),
            },
            ctx.block_map[&ctx.cur_block],
        ));

        return Ok(());
    }
    let dl = ctx.isa.data_layout();
    let sz = dl.get_size_of(ctx.types, tys[0]) as u32;
    let align = dl.get_align_of(ctx.types, tys[0]) as u32;
    let slot_id = ctx.slots.add_slot(tys[0], sz, align);
    ctx.inst_id_to_slot_id.insert(id, slot_id);
    Ok(())
}

fn lower_phi(
    ctx: &mut LoweringContext<X86_64>,
    id: InstructionId,
    ty: Type,
    args: &[ValueId],
    blocks: &[BasicBlockId],
) -> Result<()> {
    let output = new_empty_inst_output(ctx, ty, id);
    let mut operands = vec![MO::output(output.into())];
    for (arg, block) in args.iter().zip(blocks.iter()) {
        operands.push(MO::input(get_operand_for_val(ctx, ty, *arg)?));
        operands.push(MO::new(OperandData::Block(ctx.block_map[block])))
    }
    ctx.inst_seq.push(MachInstruction::new(
        InstructionData {
            opcode: Opcode::Phi,
            operands,
        },
        ctx.block_map[&ctx.cur_block],
    ));
    Ok(())
}

fn lower_bin(
    ctx: &mut LoweringContext<X86_64>,
    id: InstructionId,
    op: IrOpcode,
    ty: Type,
    args: &[ValueId],
) -> Result<()> {
    let lhs = get_vreg_for_val(ctx, ty, args[0])?;
    let output = new_empty_inst_output(ctx, ty, id);

    let insert_move = |ctx: &mut LoweringContext<X86_64>| {
        ctx.inst_seq.push(MachInstruction::new(
            InstructionData {
                opcode: Opcode::MOVrr32,
                operands: vec![MO::output(output.into()), MO::input(lhs.into())],
            },
            ctx.block_map[&ctx.cur_block],
        ))
    };

    let rhs = get_operand_for_val(ctx, ty, args[1])?;

    let data = match rhs {
        OperandData::Int32(rhs) => {
            insert_move(ctx);
            InstructionData {
                opcode: match op {
                    IrOpcode::Add => Opcode::ADDri32,
                    IrOpcode::Sub => Opcode::SUBri32,
                    // IrOpcode::Mul => Opcode::MULri32,
                    op => {
                        return Err(
                            LoweringError::Todo(format!("Unsupported opcode: {:?}", op)).into()
                        )
                    }
                },
                operands: vec![MO::input_output(output.into()), MO::new(rhs.into())],
            }
        }
        OperandData::VReg(rhs) => {
            insert_move(ctx);
            InstructionData {
                opcode: match op {
                    IrOpcode::Add => Opcode::ADDrr32,
                    IrOpcode::Sub => Opcode::SUBrr32,
                    IrOpcode::Mul => Opcode::IMULrr32,
                    op => {
                        return Err(
                            LoweringError::Todo(format!("Unsupported opcode: {:?}", op)).into()
                        )
                    }
                },
                operands: vec![MO::input_output(output.into()), MO::input(rhs.into())],
            }
        }
        e => return Err(LoweringError::Todo(format!("Unsupported operand: {:?}", e)).into()),
    };

    ctx.inst_seq
        .push(MachInstruction::new(data, ctx.block_map[&ctx.cur_block]));

    Ok(())
}

fn lower_sext(
    ctx: &mut LoweringContext<X86_64>,
    self_id: InstructionId,
    tys: &[Type; 2],
    arg: ValueId,
) -> Result<()> {
    let from = tys[0];
    let to = tys[1];
    // TODO
    assert!(from.is_i32());
    assert!(to.is_i64());

    let val = match ctx.ir_data.values[arg] {
        Value::Instruction(id) => {
            let is_mergeable_load =
                ctx.ir_data.inst_ref(id).opcode == IrOpcode::Load && from.is_i32();

            if is_mergeable_load {
                let output = new_empty_inst_output(ctx, to, self_id);
                ctx.set_output_for_inst(id, output); // Use the same output register for `load` as `sext`
                return Ok(());
            }

            get_inst_output(ctx, from, id)?
        }
        _ => {
            return Err(
                LoweringError::Todo("Sext argument must be an instruction result".into()).into(),
            )
        }
    };

    let output = new_empty_inst_output(ctx, to, self_id);

    ctx.inst_seq.push(MachInstruction::new(
        InstructionData {
            opcode: Opcode::MOVSXDr64r32,
            operands: vec![MO::output(output.into()), MO::input(val.into())],
        },
        ctx.block_map[&ctx.cur_block],
    ));

    Ok(())
}

fn lower_zext(
    ctx: &mut LoweringContext<X86_64>,
    self_id: InstructionId,
    tys: &[Type; 2],
    arg: ValueId,
) -> Result<()> {
    let from = tys[0];
    let to = tys[1];

    assert!(from.is_i8());
    assert!(to.is_i32());

    let val = get_operand_for_val(ctx, from, arg)?;
    let output = new_empty_inst_output(ctx, to, self_id);

    ctx.inst_seq.push(MachInstruction::new(
        InstructionData {
            opcode: Opcode::MOVZXr32r8,
            operands: vec![MO::output(output.into()), MO::input(val.into())],
        },
        ctx.block_map[&ctx.cur_block],
    ));

    Ok(())
}

fn lower_bitcast(
    ctx: &mut LoweringContext<X86_64>,
    self_id: InstructionId,
    tys: &[Type; 2],
    arg: ValueId,
) -> Result<()> {
    let from = tys[0];
    let to = tys[1];
    assert!({
        let from_sz = ctx.isa.data_layout().get_size_of(ctx.types, from);
        let to_sz = ctx.isa.data_layout().get_size_of(ctx.types, to);
        from_sz == to_sz && from_sz == 8
    });
    let arg = get_vreg_for_val(ctx, from, arg)?;
    let output = new_empty_inst_output(ctx, to, self_id);
    ctx.inst_seq.push(MachInstruction::new(
        InstructionData {
            opcode: Opcode::MOVrr64,
            operands: vec![MO::output(output.into()), MO::input(arg.into())],
        },
        ctx.block_map[&ctx.cur_block],
    ));
    Ok(())
}

// TODO: Refactoring.
fn lower_gep(
    ctx: &mut LoweringContext<X86_64>,
    self_id: InstructionId,
    gep: &GetElementPtr,
) -> Result<()> {
    let base = if let Value::Instruction(id) = &ctx.ir_data.values[gep.args[0]]
               && let Some(slot) = ctx.inst_id_to_slot_id.get(id).copied() {
        OperandData::Slot(slot)
    } else {
        get_operand_for_val(ctx, gep.tys[1], gep.args[0])?
    };

    // NOTE: addr = base + mul.0*idx.0 + mul.1*idx.1 + ...
    let mut indices = vec![]; // (mul, idx)
    let mut cur_ty = gep.tys[1];
    for (&arg, &arg_ty) in gep.args[1..].iter().zip(gep.tys[2..].iter()) {
        let idx = get_operand_for_val(ctx, arg_ty, arg)?;
        if cur_ty.is_struct(ctx.types) {
            let layout = ctx
                .isa
                .data_layout
                .new_struct_layout_for(ctx.types, cur_ty)
                .unwrap();
            let idx = idx.sext_as_i64().unwrap() as usize;
            let offset = layout.get_elem_offset(idx).unwrap();
            if offset != 0 {
                indices.push((1 as i64, OperandData::Int64(offset as i64)));
            }
            cur_ty = ctx.types.base().element_at(cur_ty, idx).unwrap();
        } else {
            cur_ty = ctx.types.get_element(cur_ty).unwrap();
            let sz = ctx.isa.data_layout.get_size_of(ctx.types, cur_ty) as i64;
            if let Some(idx) = idx.sext_as_i64() {
                if idx != 0 {
                    indices.push((1, OperandData::Int64(sz * idx)));
                }
            } else {
                indices.push((sz, idx));
            }
        }
    }

    let mut mem_slot = OperandData::None;
    let mut mem_imm = OperandData::None;
    let mut mem_rbase = OperandData::None;
    let mut mem_ridx = OperandData::None;
    let mut mem_mul = OperandData::None;

    if matches!(base, OperandData::Slot(_)) {
        mem_slot = base
    } else {
        mem_rbase = base
    }

    let mut simple_case = true;
    match &indices[..] {
        [] => {}
        [(1, x)] if x.sext_as_i64().is_some() => {
            mem_imm = x.to_owned();
        }
        [(_, x)] if x.sext_as_i64().is_some() => {
            unreachable!()
        }
        [(m, x)] if matches!(m, 1 | 2 | 4 | 8) => {
            mem_ridx = x.to_owned();
            mem_mul = (*m as i64).into();
        }
        _ => simple_case = false,
    }

    let ty = ctx.types.base_mut().pointer(cur_ty);
    let output = new_empty_inst_output(ctx, ty, self_id);

    if simple_case {
        ctx.inst_seq.push(MachInstruction::new(
            InstructionData {
                opcode: Opcode::LEArm64,
                operands: vec![
                    MO::output(output.into()),
                    MO::new(OperandData::MemStart),
                    MO::new(OperandData::None),
                    MO::new(mem_slot),
                    MO::new(mem_imm),
                    MO::input(mem_rbase),
                    MO::input(mem_ridx),
                    MO::new(mem_mul),
                ],
            },
            ctx.block_map[&ctx.cur_block],
        ));
        return Ok(());
    }

    ctx.inst_seq.push(MachInstruction::new(
        InstructionData {
            opcode: Opcode::LEArm64,
            operands: vec![
                MO::output(output.into()),
                MO::new(OperandData::MemStart),
                MO::new(OperandData::None),
                MO::new(mem_slot),
                MO::new(OperandData::None),
                MO::input(mem_rbase),
                MO::input(OperandData::None),
                MO::new(OperandData::None),
            ],
        },
        ctx.block_map[&ctx.cur_block],
    ));

    for (mul, idx) in indices {
        let mul_output = ctx.mach_data.vregs.add_vreg_data(ty);
        ctx.inst_seq.push(MachInstruction::new(
            InstructionData {
                opcode: Opcode::IMULrr64i32,
                operands: vec![
                    MO::output(mul_output.into()),
                    MO::input(idx.into()),
                    MO::new(OperandData::Int64(mul)),
                ],
            },
            ctx.block_map[&ctx.cur_block],
        ));
        ctx.inst_seq.push(MachInstruction::new(
            InstructionData {
                opcode: Opcode::ADDrr64,
                operands: vec![MO::output(output.into()), MO::input(mul_output.into())],
            },
            ctx.block_map[&ctx.cur_block],
        ));
    }

    Ok(())
}

fn lower_br(ctx: &mut LoweringContext<X86_64>, block: BasicBlockId) -> Result<()> {
    ctx.inst_seq.push(MachInstruction::new(
        InstructionData {
            opcode: Opcode::JMP,
            operands: vec![MO::new(OperandData::Block(ctx.block_map[&block]))],
        },
        ctx.block_map[&ctx.cur_block],
    ));
    Ok(())
}

fn lower_condbr(
    ctx: &mut LoweringContext<X86_64>,
    arg: ValueId,
    blocks: [BasicBlockId; 2],
) -> Result<()> {
    fn is_icmp<'a>(
        data: &'a IrData,
        val: &Value,
    ) -> Option<(InstructionId, &'a Type, &'a [ValueId; 2], &'a ICmpCond)> {
        match val {
            Value::Instruction(id) => {
                let inst = data.inst_ref(*id);
                match &inst.operand {
                    Operand::ICmp(ICmp { ty, args, cond }) => Some((*id, ty, args, cond)),
                    _ => None,
                }
            }
            _ => None,
        }
    }
    fn is_trunc_from_i8(data: &IrData, val: &Value) -> Option<(InstructionId, ValueId)> {
        match val {
            Value::Instruction(id) => {
                let inst = data.inst_ref(*id);
                match &inst.operand {
                    Operand::Cast(Cast {
                        arg,
                        tys: [from, to],
                    }) if inst.opcode == IrOpcode::Trunc && from.is_i8() && to.is_i1() => {
                        Some((*id, *arg))
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    let arg = ctx.ir_data.value_ref(arg);

    if let Some((icmp, ty, args, cond)) = is_icmp(ctx.ir_data, arg) {
        ctx.mark_as_merged(icmp);
        let lhs = get_vreg_for_val(ctx, *ty, args[0])?;
        let rhs = ctx.ir_data.value_ref(args[1]);
        match rhs {
            Value::Constant(ConstantValue::Int(ConstantInt::Int32(rhs))) => {
                ctx.inst_seq.push(MachInstruction::new(
                    InstructionData {
                        opcode: Opcode::CMPri32,
                        operands: vec![MO::input(lhs.into()), MO::new(rhs.into())],
                    },
                    ctx.block_map[&ctx.cur_block],
                ));
            }
            Value::Constant(ConstantValue::Null(_)) => {
                ctx.inst_seq.push(MachInstruction::new(
                    InstructionData {
                        opcode: Opcode::CMPri32,
                        operands: vec![MO::input(lhs.into()), MO::new(0.into())],
                    },
                    ctx.block_map[&ctx.cur_block],
                ));
            }
            Value::Argument(_) | Value::Instruction(_) => {
                assert!(ty.is_i32() || ty.is_i64());
                let rhs = get_operand_for_val(ctx, *ty, args[1])?;
                ctx.inst_seq.push(MachInstruction::new(
                    InstructionData {
                        opcode: Opcode::CMPrr32, // TODO: CMPrr64
                        operands: vec![MO::input(lhs.into()), MO::input(rhs.into())],
                    },
                    ctx.block_map[&ctx.cur_block],
                ));
            }
            e => return Err(LoweringError::Todo(format!("Unsupported operand: {:?}", e)).into()),
        }

        ctx.inst_seq.push(MachInstruction::new(
            InstructionData {
                opcode: match cond {
                    ICmpCond::Eq => Opcode::JE,
                    ICmpCond::Ne => Opcode::JNE,
                    ICmpCond::Sle => Opcode::JLE,
                    ICmpCond::Slt => Opcode::JL,
                    ICmpCond::Sge => Opcode::JGE,
                    ICmpCond::Sgt => Opcode::JG,
                    // ICmpCond::Ule => Opcode::JLE,
                    // ICmpCond::Ult => Opcode::JL,
                    // ICmpCond::Uge => Opcode::JGE,
                    // ICmpCond::Ugt => Opcode::JG,
                    e => {
                        return Err(LoweringError::Todo(format!(
                            "Unsupported icmp condition: {:?}",
                            e
                        ))
                        .into())
                    }
                },
                operands: vec![MO::new(OperandData::Block(ctx.block_map[&blocks[0]]))],
            },
            ctx.block_map[&ctx.cur_block],
        ));
        ctx.inst_seq.push(MachInstruction::new(
            InstructionData {
                opcode: Opcode::JMP,
                operands: vec![MO::new(OperandData::Block(ctx.block_map[&blocks[1]]))],
            },
            ctx.block_map[&ctx.cur_block],
        ));
        return Ok(());
    }

    if let Some((trunc, src)) = is_trunc_from_i8(ctx.ir_data, arg) {
        ctx.mark_as_merged(trunc);
        let lhs = get_vreg_for_val(ctx, types::I8, src)?;
        ctx.inst_seq.push(MachInstruction::new(
            InstructionData {
                opcode: Opcode::CMPri8,
                operands: vec![MO::input(lhs.into()), MO::new(0i8.into())],
            },
            ctx.block_map[&ctx.cur_block],
        ));
        ctx.inst_seq.push(MachInstruction::new(
            InstructionData {
                opcode: Opcode::JNE,
                operands: vec![MO::new(OperandData::Block(ctx.block_map[&blocks[0]]))],
            },
            ctx.block_map[&ctx.cur_block],
        ));
        ctx.inst_seq.push(MachInstruction::new(
            InstructionData {
                opcode: Opcode::JMP,
                operands: vec![MO::new(OperandData::Block(ctx.block_map[&blocks[1]]))],
            },
            ctx.block_map[&ctx.cur_block],
        ));
        return Ok(());
    }

    Err(LoweringError::Todo("Unsupported conditional br pattern".into()).into())
}

fn lower_call(
    ctx: &mut LoweringContext<X86_64>,
    id: InstructionId,
    tys: &[Type],
    args: &[ValueId],
) -> Result<()> {
    let callee = args[0];
    let name = match &ctx.ir_data.values[callee] {
        Value::Constant(ConstantValue::GlobalRef(Name::Name(name), _)) => name.clone(),
        callee => {
            return Err(LoweringError::Todo(format!("Unsupported callee: {:?}", callee)).into())
        }
    };
    log::debug!("call name: {}", name);

    let result_ty = if let Some(ty) = ctx.types.get(tys[0])
                    && let CompoundType::Function(FunctionType { ret, .. }) = &*ty {
        *ret
    } else {
        tys[0]
    };
    let result_sz = ctx.isa.data_layout().get_size_of(ctx.types, result_ty);
    let output = new_empty_inst_output(ctx, result_ty, id);

    // TODO: Refactoring.
    let gpru = RegInfo::arg_reg_list(&ctx.call_conv);
    for (gpr_used, (&arg, &ty)) in args[1..].iter().zip(tys[1..].iter()).enumerate() {
        let arg = get_operand_for_val(ctx, ty, arg)?;
        let r = gpru[gpr_used].apply(&RegClass::for_type(ctx.types, ty));
        ctx.inst_seq.push(MachInstruction::new(
            InstructionData {
                opcode: match &arg {
                    OperandData::Int64(_) => Opcode::MOVri64,
                    OperandData::Int32(_) => Opcode::MOVri32,
                    OperandData::Reg(_) => Opcode::MOVrr32, // TODO: FIXME
                    OperandData::VReg(vreg) => {
                        let ty = ctx.mach_data.vregs.type_for(*vreg);
                        let sz = ctx.isa.data_layout().get_size_of(ctx.types, ty);
                        match sz {
                            1 => Opcode::MOVrr8,
                            4 => Opcode::MOVrr32,
                            8 => Opcode::MOVrr64,
                            e => {
                                return Err(LoweringError::Todo(format!(
                                    "Unsupported argument size: {:?}",
                                    e
                                ))
                                .into())
                            }
                        }
                    }
                    e => {
                        return Err(
                            LoweringError::Todo(format!("Unsupported argument: {:?}", e)).into(),
                        )
                    }
                },
                operands: vec![MO::output(r.into()), MO::input(arg)],
            },
            ctx.block_map[&ctx.cur_block],
        ));
    }

    let result_reg: Reg = match result_sz {
        1 => GR8::AL.into(),
        4 => GR32::EAX.into(),
        8 => GR64::RAX.into(),
        _ => GR32::EAX.into(),
    };
    ctx.inst_seq.push(MachInstruction::new(
        InstructionData {
            opcode: Opcode::CALL,
            operands: vec![
                MO::implicit_output(result_reg.into()),
                MO::new(OperandData::Label(name)),
            ],
        },
        ctx.block_map[&ctx.cur_block],
    ));

    if !ctx.ir_data.users_of(id).is_empty() {
        let opcode = match result_sz {
            1 => Opcode::MOVrr8,
            4 => Opcode::MOVrr32,
            8 => Opcode::MOVrr64,
            n => todo!("Function result in {n} bytes is not supported yet"),
        };
        ctx.inst_seq.push(MachInstruction::new(
            InstructionData {
                opcode,
                operands: vec![MO::output(output.into()), MO::input(result_reg.into())],
            },
            ctx.block_map[&ctx.cur_block],
        ));
    }

    Ok(())
}

fn lower_return(ctx: &mut LoweringContext<X86_64>, arg: Option<(Type, ValueId)>) -> Result<()> {
    if let Some((ty, value)) = arg {
        let vreg = get_vreg_for_val(ctx, ty, value)?;
        let sz = ctx.isa.data_layout().get_size_of(ctx.types, ty);
        assert!(ty.is_integer() || ty.is_pointer(ctx.types));
        let (reg, opcode) = match sz {
            4 => (GR32::EAX.into(), Opcode::MOVrr32),
            8 => (GR64::RAX.into(), Opcode::MOVrr64),
            _ => todo!(),
        };
        ctx.inst_seq.push(MachInstruction::new(
            InstructionData {
                opcode,
                operands: vec![MO::output(OperandData::Reg(reg)), MO::input(vreg.into())],
            },
            ctx.block_map[&ctx.cur_block],
        ));
    }
    ctx.inst_seq.push(MachInstruction::new(
        InstructionData {
            opcode: Opcode::RET,
            operands: vec![],
        },
        ctx.block_map[&ctx.cur_block],
    ));
    Ok(())
}

fn get_inst_output(ctx: &mut LoweringContext<X86_64>, ty: Type, id: InstructionId) -> Result<VReg> {
    if let Some(vreg) = ctx.inst_id_to_vreg.get(&id) {
        return Ok(*vreg);
    }

    if ctx.ir_data.inst_ref(id).parent != ctx.cur_block {
        // The instruction indexed as `id` must be placed in another basic block
        let vreg = new_empty_inst_output(ctx, ty, id);
        return Ok(vreg);
    }

    let inst = ctx.ir_data.inst_ref(id);
    lower(ctx, inst)?;

    Ok(new_empty_inst_output(ctx, ty, id))
}

fn new_empty_inst_output(ctx: &mut LoweringContext<X86_64>, ty: Type, id: InstructionId) -> VReg {
    if let Some(vreg) = ctx.inst_id_to_vreg.get(&id) {
        return *vreg;
    }
    let vreg = ctx.mach_data.vregs.add_vreg_data(ty);
    ctx.inst_id_to_vreg.insert(id, vreg);
    vreg
}

fn get_operand_for_val(
    ctx: &mut LoweringContext<X86_64>,
    ty: Type,
    val: ValueId,
) -> Result<OperandData> {
    match ctx.ir_data.values[val] {
        Value::Instruction(id) => Ok(get_inst_output(ctx, ty, id)?.into()),
        Value::Argument(ref a) => Ok(ctx.arg_idx_to_vreg[&a.nth].into()),
        Value::Constant(ref konst) => get_operand_for_const(ctx, ty, konst),
        ref e => Err(LoweringError::Todo(format!("Unsupported value: {:?}", e)).into()),
    }
}

fn get_operand_for_const(
    ctx: &mut LoweringContext<X86_64>,
    ty: Type,
    konst: &ConstantValue,
) -> Result<OperandData> {
    match konst {
        ConstantValue::Int(ConstantInt::Int32(i)) => Ok(OperandData::Int32(*i)),
        ConstantValue::Int(ConstantInt::Int64(i)) => Ok(OperandData::Int64(*i)),
        ConstantValue::Expr(ConstantExpr::GetElementPtr {
            inbounds: _,
            tys: _,
            ref args,
        }) => {
            // TODO: Just refactor this.
            assert!(ty.is_pointer(ctx.types));
            assert!(matches!(args[0], ConstantValue::GlobalRef(_, _)));
            let all_indices_0 = args[1..]
                .iter()
                .all(|arg| matches!(arg, ConstantValue::Int(i) if i.is_zero()));
            assert!(all_indices_0);
            let src = OperandData::GlobalAddress(args[0].as_global_ref().as_string().clone());
            let dst = ctx.mach_data.vregs.add_vreg_data(ty);
            ctx.inst_seq.push(MachInstruction::new(
                InstructionData {
                    opcode: Opcode::MOVri64,
                    operands: vec![MO::output(dst.into()), MO::new(src)],
                },
                ctx.block_map[&ctx.cur_block],
            ));
            Ok(dst.into())
        }
        ConstantValue::Expr(ConstantExpr::Bitcast {
            tys: [from, to],
            arg,
        }) => {
            assert!(from.is_pointer(ctx.types));
            assert!(to.is_pointer(ctx.types));
            get_operand_for_const(ctx, *to, arg)
        }
        ConstantValue::GlobalRef(ref name, ty) => {
            assert!(ty.is_pointer(ctx.types));
            let addr = ctx.mach_data.vregs.add_vreg_data(*ty);
            let src = OperandData::GlobalAddress(name.to_string().unwrap().to_owned());
            ctx.inst_seq.push(MachInstruction::new(
                InstructionData {
                    opcode: Opcode::MOVri64,
                    operands: vec![MO::output(addr.into()), MO::new(src)],
                },
                ctx.block_map[&ctx.cur_block],
            ));
            Ok(addr.into())
        }
        ConstantValue::Null(ty) => {
            assert!(ty.is_pointer(ctx.types));
            let addr = ctx.mach_data.vregs.add_vreg_data(*ty);
            ctx.inst_seq.push(MachInstruction::new(
                InstructionData {
                    opcode: Opcode::MOVri64,
                    operands: vec![MO::output(addr.into()), MO::new(0.into())],
                },
                ctx.block_map[&ctx.cur_block],
            ));
            Ok(addr.into())
        }
        e => todo!("{:?}", e),
    }
}

fn get_vreg_for_val(ctx: &mut LoweringContext<X86_64>, ty: Type, val: ValueId) -> Result<VReg> {
    match get_operand_for_val(ctx, ty, val)? {
        OperandData::Int32(i) => {
            let output = ctx.mach_data.vregs.add_vreg_data(ty);
            ctx.inst_seq.push(MachInstruction::new(
                InstructionData {
                    opcode: Opcode::MOVri32,
                    operands: vec![MO::output(output.into()), MO::new(i.into())],
                },
                ctx.block_map[&ctx.cur_block],
            ));
            Ok(output)
        }
        OperandData::VReg(vr) => Ok(vr),
        e => Err(LoweringError::Todo(format!("Unsupported operand: {:?}", e)).into()),
    }
}
