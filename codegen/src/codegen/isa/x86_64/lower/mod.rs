pub mod load;
pub mod store;

use crate::codegen::{
    function::instruction::Instruction as MachInstruction,
    isa::x86_64::{
        instruction::{InstructionData, Opcode, Operand as MO, OperandData},
        register::{RegClass, RegInfo, GR32},
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
    types::Type,
    value::{ConstantExpr, ConstantInt, ConstantValue, Value, ValueId},
};

#[derive(Clone, Copy)]
pub struct Lower {}

impl Default for Lower {
    fn default() -> Self {
        Lower {}
    }
}

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
        _ => Err(LoweringError::Todo.into()),
    }
}

fn lower_alloca(
    ctx: &mut LoweringContext<X86_64>,
    id: InstructionId,
    tys: &[Type],
    _num_elements: &ConstantValue,
    _align: u32,
) -> Result<()> {
    let slot_id = ctx
        .slots
        .add_slot(tys[0], X86_64::type_size(ctx.types, tys[0]));
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
                    _ => return Err(LoweringError::Todo.into()),
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
                    _ => return Err(LoweringError::Todo.into()),
                },
                operands: vec![MO::input_output(output.into()), MO::input(rhs.into())],
            }
        }
        _ => return Err(LoweringError::Todo.into()),
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
        _ => return Err(LoweringError::Todo.into()),
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
    ) -> Option<(&'a Type, &'a [ValueId; 2], &'a ICmpCond)> {
        match val {
            Value::Instruction(id) => {
                let inst = data.inst_ref(*id);
                match &inst.operand {
                    Operand::ICmp(ICmp { ty, args, cond }) => Some((ty, args, cond)),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    let arg = ctx.ir_data.value_ref(arg);

    if let Some((ty, args, cond)) = is_icmp(ctx.ir_data, arg) {
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
            _ => return Err(LoweringError::Todo.into()),
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
                    _ => return Err(LoweringError::Todo.into()),
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

    Err(LoweringError::Todo.into())
}

fn lower_call(
    ctx: &mut LoweringContext<X86_64>,
    id: InstructionId,
    tys: &[Type],
    args: &[ValueId],
) -> Result<()> {
    let output = new_empty_inst_output(ctx, tys[0], id);

    let gpru = RegInfo::arg_reg_list(&ctx.call_conv);
    for (gpr_used, (&arg, &ty)) in args[1..].iter().zip(tys[1..].iter()).enumerate() {
        let arg = val_to_operand_data(ctx, ty, arg)?;
        let r = gpru[gpr_used].apply(&RegClass::for_type(ctx.types, ty));
        ctx.inst_seq.push(MachInstruction::new(
            InstructionData {
                opcode: match &arg {
                    OperandData::Int32(_) => Opcode::MOVri32,
                    OperandData::VReg(_) | OperandData::Reg(_) => Opcode::MOVrr32,
                    _ => return Err(LoweringError::Todo.into()),
                },
                operands: vec![MO::output(r.into()), MO::input(arg)],
            },
            ctx.block_map[&ctx.cur_block],
        ));
    }

    let name = match &ctx.ir_data.values[args[0]] {
        Value::Constant(ConstantValue::GlobalRef(Name::Name(name), _)) => name.clone(),
        _ => return Err(LoweringError::Todo.into()),
    };
    let result_reg: Reg = GR32::EAX.into(); // TODO: do not hard code
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

    let inst = ctx.ir_data.inst_ref(id);

    if inst.opcode.has_side_effects() {
        let vreg = new_empty_inst_output(ctx, ty, id);
        Ok(vreg)
    } else {
        // TODO: What about instruction scheduling?
        lower(ctx, inst)?;
        get_or_generate_inst_output(ctx, ty, id)
    }
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
        Value::Constant(ConstantValue::Int(ConstantInt::Int32(i))) => Ok(OperandData::Int32(i)),
        Value::Constant(ConstantValue::Expr(ConstantExpr::GetElementPtr {
            inbounds: _,
            tys: _,
            ref args,
        })) => {
            // TODO: Split up into functions
            assert!(ty.is_pointer(&ctx.types));
            assert!(matches!(args[0], ConstantValue::GlobalRef(_, _)));
            let all_indices_0 = args[1..]
                .iter()
                .all(|arg| matches!(arg, ConstantValue::Int(ConstantInt::Int64(0))));
            assert!(all_indices_0);
            let src = OperandData::GlobalAddress(args[0].as_global_ref().as_string().clone());
            let dst = ctx.mach_data.vregs.add_vreg_data(ty);
            ctx.inst_seq.push(MachInstruction::new(
                InstructionData {
                    opcode: Opcode::MOVri32, // TODO: MOVri64 is correct
                    operands: vec![MO::output(dst.into()), MO::new(src)],
                },
                ctx.block_map[&ctx.cur_block],
            ));
            Ok(dst.into())
        }
        _ => Err(LoweringError::Todo.into()),
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
        _ => Err(LoweringError::Todo.into()),
    }
}
