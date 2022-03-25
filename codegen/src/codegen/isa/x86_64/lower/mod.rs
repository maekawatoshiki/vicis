pub mod load;
pub mod store;

use crate::codegen::{
    function::instruction::Instruction as MachInstruction,
    isa::x86_64::{
        instruction::{InstructionData, Opcode, Operand as MO, OperandData},
        register::{RegClass, RegInfo, GR32, GR64},
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
            Alloca, Br, Call, Cast, CondBr, ICmp, ICmpCond, Instruction as IrInstruction,
            InstructionId, IntBinary, Load, Opcode as IrOpcode, Operand, Phi, Ret, Store,
        },
        Parameter,
    },
    module::name::Name,
    types::{CompoundType, FunctionType, Type},
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
            assert!(ty.is_i32());
            let output = ctx.mach_data.vregs.add_vreg_data(*ty);
            ctx.inst_seq.push(MachInstruction::new(
                InstructionData {
                    opcode: Opcode::MOVrr32,
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
    let sz = ctx.isa.data_layout().get_size_of(ctx.types, tys[0]) as u32;
    let slot_id = ctx.slots.add_slot(tys[0], sz);
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
        operands.push(MO::input(val_to_operand_data(ctx, ty, *arg)?));
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
    let lhs = val_to_vreg(ctx, ty, args[0])?;
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

    let rhs = val_to_operand_data(ctx, ty, args[1])?;

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

            get_or_generate_inst_output(ctx, from, id)?
        }
        _ => {
            return Err(
                LoweringError::Todo(format!("Sext argument must be an instruction result")).into(),
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

    let arg = ctx.ir_data.value_ref(arg);

    if let Some((icmp, ty, args, cond)) = is_icmp(ctx.ir_data, arg) {
        ctx.mark_as_merged(icmp);
        let lhs = val_to_vreg(ctx, *ty, args[0])?;
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
            Value::Instruction(id) => {
                assert!(ty.is_i32());
                let rhs = get_or_generate_inst_output(ctx, *ty, *id)?;
                ctx.inst_seq.push(MachInstruction::new(
                    InstructionData {
                        opcode: Opcode::CMPrr32,
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

    Err(LoweringError::Todo("Unsupported conditional br pattern".into()).into())
}

fn lower_call(
    ctx: &mut LoweringContext<X86_64>,
    id: InstructionId,
    tys: &[Type],
    args: &[ValueId],
) -> Result<()> {
    let result_ty = if let Some(ty) = ctx.types.get(tys[0])
                    && let CompoundType::Function(FunctionType { ret, .. }) = &*ty {
        *ret
    } else {
        tys[0]
    };
    let result_sz = ctx.isa.data_layout().get_size_of(ctx.types, result_ty);
    let output = new_empty_inst_output(ctx, result_ty, id);

    let gpru = RegInfo::arg_reg_list(&ctx.call_conv);
    for (gpr_used, (&arg, &ty)) in args[1..].iter().zip(tys[1..].iter()).enumerate() {
        let arg = val_to_operand_data(ctx, ty, arg)?;
        let r = gpru[gpr_used].apply(&RegClass::for_type(ctx.types, ty));
        ctx.inst_seq.push(MachInstruction::new(
            InstructionData {
                opcode: match &arg {
                    OperandData::Int32(_) => Opcode::MOVri32,
                    OperandData::Reg(_) => Opcode::MOVrr32, // TODO: FIXME
                    OperandData::VReg(vreg) => {
                        let ty = ctx.mach_data.vregs.type_for(*vreg);
                        let sz = ctx.isa.data_layout().get_size_of(ctx.types, ty);
                        match sz {
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

    let callee = args[0];
    let name = match &ctx.ir_data.values[callee] {
        Value::Constant(ConstantValue::GlobalRef(Name::Name(name), _)) => name.clone(),
        e => return Err(LoweringError::Todo(format!("Unsupported callee: {:?}", e)).into()),
    };
    let result_reg: Reg = match result_sz {
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
        match result_sz {
            4 => Opcode::MOVrr32,
            8 => Opcode::MOVrr64,
            _ => todo!("Function results less than 32 bit are not supported yet"),
        };
        ctx.inst_seq.push(MachInstruction::new(
            InstructionData {
                opcode: Opcode::MOVrr32,
                operands: vec![MO::output(output.into()), MO::input(result_reg.into())],
            },
            ctx.block_map[&ctx.cur_block],
        ));
    }

    Ok(())
}

fn lower_return(ctx: &mut LoweringContext<X86_64>, arg: Option<(Type, ValueId)>) -> Result<()> {
    if let Some((ty, value)) = arg {
        let vreg = val_to_vreg(ctx, ty, value)?;
        assert!(ty.is_i32());
        ctx.inst_seq.push(MachInstruction::new(
            InstructionData {
                opcode: Opcode::MOVrr32,
                operands: vec![
                    MO::output(OperandData::Reg(GR32::EAX.into())),
                    MO::input(vreg.into()),
                ],
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

// Get instruction output.
// If the instruction is not placed in any basic block, place it in the current block.
// If the instruction must be placed in another block except the current block(, which means
// the instruction output must live out from its parent basic block to the current block),
// just create a new virtual register to store the instruction output.
fn get_or_generate_inst_output(
    ctx: &mut LoweringContext<X86_64>,
    ty: Type,
    id: InstructionId,
) -> Result<VReg> {
    if let Some(vreg) = ctx.inst_id_to_vreg.get(&id) {
        return Ok(*vreg);
    }

    if ctx.ir_data.inst_ref(id).parent != ctx.cur_block {
        // The instruction indexed as `id` must be placed in another basic block
        let vreg = new_empty_inst_output(ctx, ty, id);
        return Ok(vreg);
    }

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

fn val_to_operand_data(
    ctx: &mut LoweringContext<X86_64>,
    ty: Type,
    val: ValueId,
) -> Result<OperandData> {
    match ctx.ir_data.values[val] {
        Value::Instruction(id) => Ok(get_or_generate_inst_output(ctx, ty, id)?.into()),
        Value::Argument(ref a) => Ok(ctx.arg_idx_to_vreg[&a.nth].into()),
        Value::Constant(ref konst) => const_to_operand_data(ctx, ty, konst),
        ref e => todo!("{:?}", e),
        // _ => Err(LoweringError::Todo.into()),
    }
}

fn const_to_operand_data(
    ctx: &mut LoweringContext<X86_64>,
    ty: Type,
    konst: &ConstantValue,
) -> Result<OperandData> {
    match konst {
        ConstantValue::Int(ConstantInt::Int32(i)) => Ok(OperandData::Int32(*i)),
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
            const_to_operand_data(ctx, *to, arg)
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
        e => todo!("{:?}", e),
    }
}

fn val_to_vreg(ctx: &mut LoweringContext<X86_64>, ty: Type, val: ValueId) -> Result<VReg> {
    match val_to_operand_data(ctx, ty, val)? {
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
