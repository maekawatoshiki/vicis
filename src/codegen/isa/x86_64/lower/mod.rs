pub mod load;
pub mod store;

use crate::codegen::{
    function::instruction::Instruction as MachInstruction,
    isa::x86_64::{
        instruction::{InstructionData, Opcode, Operand as MOperand, OperandData},
        register::{RegClass, RegInfo, GR32},
        X86_64,
    },
    isa::TargetIsa,
    lower::{Lower as LowerTrait, LoweringContext, LoweringError},
    register::{Reg, RegisterClass, RegisterInfo, VReg},
};
use crate::ir::{
    function::{
        basic_block::BasicBlockId,
        data::Data as IrData,
        instruction::{
            ICmpCond, Instruction as IrInstruction, InstructionId, Opcode as IrOpcode, Operand,
        },
        Parameter,
    },
    module::name::Name,
    types::{Type, TypeId},
    value::{ConstantData, ConstantExpr, ConstantInt, Value, ValueId},
};
use anyhow::Result;
use load::lower_load;
use store::lower_store;

#[derive(Clone, Copy)]
pub struct Lower {}

impl Lower {
    pub fn new() -> Self {
        Lower {}
    }
}

impl LowerTrait<X86_64> for Lower {
    fn lower(ctx: &mut LoweringContext<X86_64>, inst: &IrInstruction) -> Result<()> {
        lower(ctx, inst)
    }

    fn copy_args_to_vregs(ctx: &mut LoweringContext<X86_64>, params: &[Parameter]) -> Result<()> {
        let mut gpr_used = 0;
        let args = RegInfo::arg_reg_list(&ctx.call_conv);
        for (i, Parameter { name: _, ty }) in params.iter().enumerate() {
            let reg = args[gpr_used].apply(&RegClass::for_type(ctx.types, *ty));
            gpr_used += 1;
            debug!(reg);
            // Copy reg to new vreg
            assert!(*ctx.types.get(*ty) == Type::Int(32));
            let output = ctx.mach_data.vregs.add_vreg_data(*ty);
            ctx.inst_seq.push(MachInstruction::new(
                InstructionData {
                    opcode: Opcode::MOVrr32,
                    operands: vec![MOperand::output(output.into()), MOperand::input(reg.into())],
                },
                ctx.block_map[&ctx.cur_block],
            ));
            ctx.arg_idx_to_vreg.insert(i, output);
        }
        Ok(())
    }
}

fn lower(ctx: &mut LoweringContext<X86_64>, inst: &IrInstruction) -> Result<()> {
    match inst.operand {
        Operand::Alloca {
            ref tys,
            ref num_elements,
            align,
        } => lower_alloca(ctx, inst.id.unwrap(), tys, num_elements, align),
        Operand::Phi {
            ty,
            ref args,
            ref blocks,
        } => lower_phi(ctx, inst.id.unwrap(), ty, args, blocks),
        Operand::Load {
            ref tys,
            addr,
            align,
        } => lower_load(ctx, inst.id.unwrap(), tys, addr, align),
        Operand::Store {
            ref tys,
            ref args,
            align,
        } => lower_store(ctx, tys, args, align),
        Operand::IntBinary { ty, ref args, .. } => {
            lower_bin(ctx, inst.id.unwrap(), inst.opcode, ty, args)
        }
        Operand::Cast { ref tys, arg } if inst.opcode == IrOpcode::Sext => {
            lower_sext(ctx, inst.id.unwrap(), tys, arg)
        }
        Operand::Br { block } => lower_br(ctx, block),
        Operand::CondBr { arg, blocks } => lower_condbr(ctx, arg, blocks),
        Operand::Call {
            ref args, ref tys, ..
        } => lower_call(ctx, inst.id.unwrap(), tys, args),
        Operand::Ret { val: None, .. } => Err(LoweringError::Todo.into()),
        Operand::Ret { val: Some(val), ty } => lower_return(ctx, ty, val),
        _ => Err(LoweringError::Todo.into()),
    }
}

fn lower_alloca(
    ctx: &mut LoweringContext<X86_64>,
    id: InstructionId,
    tys: &[TypeId],
    _num_elements: &ConstantData,
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
    ty: TypeId,
    args: &[ValueId],
    blocks: &[BasicBlockId],
) -> Result<()> {
    let output = new_empty_inst_output(ctx, ty, id);
    let mut operands = vec![MOperand::output(output.into())];
    for (arg, block) in args.iter().zip(blocks.iter()) {
        operands.push(MOperand::input(val_to_operand_data(ctx, ty, *arg)?));
        operands.push(MOperand::new(OperandData::Block(ctx.block_map[block])))
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
    ty: TypeId,
    args: &[ValueId],
) -> Result<()> {
    let lhs = val_to_vreg(ctx, ty, args[0])?;
    let output = new_empty_inst_output(ctx, ty, id);

    let insert_move = |ctx: &mut LoweringContext<X86_64>| {
        ctx.inst_seq.push(MachInstruction::new(
            InstructionData {
                opcode: Opcode::MOVrr32,
                operands: vec![MOperand::output(output.into()), MOperand::input(lhs.into())],
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
                operands: vec![
                    MOperand::input_output(output.into()),
                    MOperand::new(rhs.into()),
                ],
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
                operands: vec![
                    MOperand::input_output(output.into()),
                    MOperand::input(rhs.into()),
                ],
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
    id: InstructionId,
    tys: &[TypeId; 2],
    arg: ValueId,
) -> Result<()> {
    if ctx.is_merged(id) {
        return Ok(());
    }

    let from = tys[0];
    let to = tys[1];
    assert_eq!(*ctx.types.get(from), Type::Int(32));
    assert_eq!(*ctx.types.get(to), Type::Int(64));

    let val = match ctx.ir_data.values[arg] {
        Value::Instruction(id) => get_or_generate_inst_output(ctx, from, id)?,
        _ => return Err(LoweringError::Todo.into()),
    };

    let output = new_empty_inst_output(ctx, to, id);

    ctx.inst_seq.push(MachInstruction::new(
        InstructionData {
            opcode: Opcode::MOVSXDr64r32,
            operands: vec![MOperand::output(output.into()), MOperand::input(val.into())],
        },
        ctx.block_map[&ctx.cur_block],
    ));

    Ok(())
}

fn lower_br(ctx: &mut LoweringContext<X86_64>, block: BasicBlockId) -> Result<()> {
    ctx.inst_seq.push(MachInstruction::new(
        InstructionData {
            opcode: Opcode::JMP,
            operands: vec![MOperand::new(OperandData::Block(ctx.block_map[&block]))],
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
    ) -> Option<(&'a TypeId, &'a [ValueId; 2], &'a ICmpCond)> {
        match val {
            Value::Instruction(id) => {
                let inst = data.inst_ref(*id);
                match &inst.operand {
                    Operand::ICmp { ty, args, cond } => return Some((ty, args, cond)),
                    _ => return None,
                }
            }
            _ => return None,
        }
    }

    let arg = ctx.ir_data.value_ref(arg);

    if let Some((ty, args, cond)) = is_icmp(ctx.ir_data, arg) {
        let lhs = val_to_vreg(ctx, *ty, args[0])?;
        let rhs = ctx.ir_data.value_ref(args[1]);
        match rhs {
            Value::Constant(ConstantData::Int(ConstantInt::Int32(rhs))) => {
                ctx.inst_seq.push(MachInstruction::new(
                    InstructionData {
                        opcode: Opcode::CMPri32,
                        operands: vec![MOperand::input(lhs.into()), MOperand::new(rhs.into())],
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
                operands: vec![MOperand::new(OperandData::Block(ctx.block_map[&blocks[0]]))],
            },
            ctx.block_map[&ctx.cur_block],
        ));
        ctx.inst_seq.push(MachInstruction::new(
            InstructionData {
                opcode: Opcode::JMP,
                operands: vec![MOperand::new(OperandData::Block(ctx.block_map[&blocks[1]]))],
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
    tys: &[TypeId],
    args: &[ValueId],
) -> Result<()> {
    let output = new_empty_inst_output(ctx, tys[0], id);

    let gpru = RegInfo::arg_reg_list(&ctx.call_conv);
    let mut gpr_used = 0;
    for (&arg, &ty) in args[1..].iter().zip(tys[1..].iter()) {
        let arg = val_to_operand_data(ctx, ty, arg)?;
        let r = gpru[gpr_used].apply(&RegClass::for_type(ctx.types, ty));
        gpr_used += 1;
        ctx.inst_seq.push(MachInstruction::new(
            InstructionData {
                opcode: match &arg {
                    OperandData::Int32(_) => Opcode::MOVri32,
                    OperandData::VReg(_) | OperandData::Reg(_) => Opcode::MOVrr32,
                    _ => return Err(LoweringError::Todo.into()),
                },
                operands: vec![MOperand::output(r.into()), MOperand::input(arg.into())],
            },
            ctx.block_map[&ctx.cur_block],
        ));
    }

    let name = match &ctx.ir_data.values[args[0]] {
        Value::Constant(ConstantData::GlobalRef(Name::Name(name))) => name.clone(),
        _ => return Err(LoweringError::Todo.into()),
    };
    let result_reg: Reg = GR32::EAX.into(); // TODO: do not hard code
    ctx.inst_seq.push(MachInstruction::new(
        InstructionData {
            opcode: Opcode::CALL,
            operands: vec![
                MOperand::implicit_output(result_reg.into()),
                MOperand::new(OperandData::Label(name)),
            ],
        },
        ctx.block_map[&ctx.cur_block],
    ));

    if ctx.ir_data.users_of(id).len() > 0 {
        ctx.inst_seq.push(MachInstruction::new(
            InstructionData {
                opcode: Opcode::MOVrr32,
                operands: vec![
                    MOperand::output(output.into()),
                    MOperand::input(result_reg.into()),
                ],
            },
            ctx.block_map[&ctx.cur_block],
        ));
    }

    Ok(())
}

fn lower_return(ctx: &mut LoweringContext<X86_64>, ty: TypeId, value: ValueId) -> Result<()> {
    let vreg = val_to_vreg(ctx, ty, value)?;
    assert!(*ctx.types.get(ty) == Type::Int(32));
    ctx.inst_seq.push(MachInstruction::new(
        InstructionData {
            opcode: Opcode::MOVrr32,
            operands: vec![
                MOperand::output(OperandData::Reg(GR32::EAX.into())),
                MOperand::input(vreg.into()),
            ],
        },
        ctx.block_map[&ctx.cur_block],
    ));
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
    ty: TypeId,
    id: InstructionId,
) -> Result<VReg> {
    if let Some(vreg) = ctx.inst_id_to_vreg.get(&id) {
        return Ok(*vreg);
    }

    if ctx.ir_data.inst_ref(id).parent != ctx.cur_block {
        // The instruction indexed as `id` must be placed in another basic block
        let v = ctx.mach_data.vregs.add_vreg_data(ty);
        ctx.inst_id_to_vreg.insert(id, v);
        return Ok(v);
    }

    // TODO: What about instruction scheduling?
    lower(ctx, ctx.ir_data.inst_ref(id))?;
    get_or_generate_inst_output(ctx, ty, id)
}

fn new_empty_inst_output(ctx: &mut LoweringContext<X86_64>, ty: TypeId, id: InstructionId) -> VReg {
    if let Some(vreg) = ctx.inst_id_to_vreg.get(&id) {
        return *vreg;
    }
    let vreg = ctx.mach_data.vregs.add_vreg_data(ty);
    ctx.inst_id_to_vreg.insert(id, vreg);
    vreg
}

fn val_to_operand_data(
    ctx: &mut LoweringContext<X86_64>,
    ty: TypeId,
    val: ValueId,
) -> Result<OperandData> {
    match ctx.ir_data.values[val] {
        Value::Instruction(id) => Ok(get_or_generate_inst_output(ctx, ty, id)?.into()),
        Value::Argument(idx) => Ok(ctx.arg_idx_to_vreg[&idx].into()),
        Value::Constant(ConstantData::Int(ConstantInt::Int32(i))) => Ok(OperandData::Int32(i)),
        Value::Constant(ConstantData::Expr(ConstantExpr::GetElementPtr {
            inbounds: _,
            tys: _,
            ref args,
        })) => {
            // TODO: Split up into functions
            assert!(matches!(&*ctx.types.get(ty), Type::Pointer(_)));
            assert!(matches!(args[0], ConstantData::GlobalRef(_)));
            let all_indices_0 = args[1..]
                .iter()
                .all(|arg| matches!(arg, ConstantData::Int(ConstantInt::Int64(0))));
            assert!(all_indices_0);
            let src = OperandData::GlobalAddress(args[0].as_global_ref().as_string().clone());
            let dst = ctx.mach_data.vregs.add_vreg_data(ty);
            ctx.inst_seq.push(MachInstruction::new(
                InstructionData {
                    opcode: Opcode::MOVri32, // TODO: MOVri64 is correct
                    operands: vec![MOperand::output(dst.into()), MOperand::new(src.into())],
                },
                ctx.block_map[&ctx.cur_block],
            ));
            Ok(dst.into())
        }
        _ => Err(LoweringError::Todo.into()),
    }
}

fn val_to_vreg(ctx: &mut LoweringContext<X86_64>, ty: TypeId, val: ValueId) -> Result<VReg> {
    match val_to_operand_data(ctx, ty, val)? {
        OperandData::Int32(i) => {
            let output = ctx.mach_data.vregs.add_vreg_data(ty);
            ctx.inst_seq.push(MachInstruction::new(
                InstructionData {
                    opcode: Opcode::MOVri32,
                    operands: vec![MOperand::output(output.into()), MOperand::new(i.into())],
                },
                ctx.block_map[&ctx.cur_block],
            ));
            Ok(output)
        }
        OperandData::VReg(vr) => Ok(vr),
        _ => Err(LoweringError::Todo.into()),
    }
}
