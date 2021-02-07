use crate::codegen::{
    function::instruction::Instruction as MachInstruction,
    lower::{Lower as LowerTrait, LoweringContext},
    register::{Reg, RegisterClass, RegisterInfo, VReg},
    target::x86_64::{
        instruction::{InstructionData, Opcode, Operand as MOperand, OperandData},
        register::{RegClass, RegInfo, GR32},
        X86_64,
    },
    target::Target,
};
use crate::ir::{
    function::{
        basic_block::BasicBlockId,
        instruction::{
            ICmpCond, Instruction as IrInstruction, InstructionId, Opcode as IrOpcode, Operand,
        },
        Data as IrData, Parameter,
    },
    module::name::Name,
    types::{Type, TypeId},
    value::{ConstantData, ConstantInt, Value, ValueId},
};

#[derive(Clone, Copy)]
pub struct Lower {}

impl Lower {
    pub fn new() -> Self {
        Lower {}
    }
}

impl LowerTrait<X86_64> for Lower {
    fn lower(ctx: &mut LoweringContext<X86_64>, inst: &IrInstruction) {
        lower(ctx, inst)
    }

    fn copy_args_to_vregs(ctx: &mut LoweringContext<X86_64>, params: &[Parameter]) {
        let mut gpr_used = 0;
        let args = RegInfo::arg_reg_list(&ctx.call_conv);
        for (i, Parameter { name: _, ty }) in params.iter().enumerate() {
            let reg = args[gpr_used].apply(&RegClass::for_type(ctx.types, *ty));
            gpr_used += 1;
            debug!(reg);
            // Copy reg to new vreg
            assert!(*ctx.types.get(*ty) == Type::Int(32));
            let output = ctx.vregs.add_vreg_data(*ty);
            ctx.inst_seq.push(MachInstruction {
                id: None,
                data: InstructionData {
                    opcode: Opcode::MOVrr32,
                    operands: vec![MOperand::output(output.into()), MOperand::input(reg.into())],
                },
            });
            ctx.arg_idx_to_vreg.insert(i, output);
        }
    }
}

fn lower(ctx: &mut LoweringContext<X86_64>, inst: &IrInstruction) {
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
        Operand::IntBinary { ty, ref args, .. } => lower_add(ctx, inst.id.unwrap(), ty, args),
        Operand::Cast { ref tys, arg } if inst.opcode == IrOpcode::Sext => {
            lower_sext(ctx, inst.id.unwrap(), tys, arg)
        }
        Operand::Br { block } => lower_br(ctx, block),
        Operand::CondBr { arg, blocks } => lower_condbr(ctx, arg, blocks),
        Operand::Call { ref args, ref tys } => lower_call(ctx, inst.id.unwrap(), tys, args),
        Operand::Ret { val: None, .. } => todo!(),
        Operand::Ret { val: Some(val), ty } => lower_return(ctx, ty, val),
        _ => todo!(),
    }
}

fn lower_alloca(
    ctx: &mut LoweringContext<X86_64>,
    id: InstructionId,
    tys: &[TypeId],
    _num_elements: &ConstantData,
    _align: u32,
) {
    let slot_id = ctx
        .slots
        .add_slot(tys[0], X86_64::type_size(ctx.types, tys[0]));
    ctx.inst_id_to_slot_id.insert(id, slot_id);
}

fn lower_phi(
    ctx: &mut LoweringContext<X86_64>,
    id: InstructionId,
    ty: TypeId,
    args: &[ValueId],
    blocks: &[BasicBlockId],
) {
    let output = new_empty_inst_output(ctx, ty, id);
    let mut operands = vec![MOperand::output(output.into())];
    for (arg, block) in args.iter().zip(blocks.iter()) {
        operands.push(MOperand::input(val_to_operand_data(ctx, ty, *arg)));
        operands.push(MOperand::new(OperandData::Block(ctx.block_map[block])))
    }
    ctx.inst_seq.push(MachInstruction::new(InstructionData {
        opcode: Opcode::Phi,
        operands,
    }));
}

fn lower_load(
    ctx: &mut LoweringContext<X86_64>,
    id: InstructionId,
    tys: &[TypeId],
    addr: ValueId,
    _align: u32,
) {
    let mut slot = None;

    match ctx.ir_data.value_ref(addr) {
        Value::Instruction(id) => {
            if let Some(slot_id) = ctx.inst_id_to_slot_id.get(id) {
                slot = Some(*slot_id);
            }
        }
        _ => todo!(),
    }

    if let Some(slot) = slot {
        if matches!(&*ctx.types.get(tys[0]), Type::Int(32)) {
            let output = new_empty_inst_output(ctx, tys[0], id);
            ctx.inst_seq.push(MachInstruction::new(InstructionData {
                opcode: Opcode::MOVrm32,
                operands: vec![
                    MOperand::output(output.into()),
                    MOperand::new(OperandData::MemStart),
                    MOperand::new(OperandData::Slot(slot)),
                    MOperand::new(OperandData::None),
                    MOperand::input(OperandData::None),
                    MOperand::input(OperandData::None),
                    MOperand::new(OperandData::None),
                ],
            }));
            return;
        }
    }

    todo!()
}

fn lower_store(ctx: &mut LoweringContext<X86_64>, tys: &[TypeId], args: &[ValueId], _align: u32) {
    let mut slot = None;

    match ctx.ir_data.value_ref(args[1]) {
        Value::Instruction(id) => {
            if let Some(slot_id) = ctx.inst_id_to_slot_id.get(id) {
                // Maybe Alloca
                slot = Some(*slot_id);
            } else {
                if ctx.ir_data.instructions[*id].opcode == IrOpcode::GetElementPtr {
                    return lower_store_gep(ctx, tys, args, _align, *id);
                }
            }
        }
        _ => todo!(),
    }

    let mut const_int = None;
    let mut inst = None;

    match ctx.ir_data.value_ref(args[0]) {
        Value::Constant(ConstantData::Int(int)) => const_int = Some(*int),
        Value::Instruction(id) => inst = Some(*id),
        _ => {}
    }

    match (slot, inst) {
        (Some(slot), Some(id)) => {
            let inst = get_or_generate_inst_output(ctx, tys[0], id);
            ctx.inst_seq
                .append(&mut vec![MachInstruction::new(InstructionData {
                    opcode: Opcode::MOVmr32,
                    operands: vec![
                        MOperand::new(OperandData::MemStart),
                        MOperand::new(OperandData::Slot(slot)),
                        MOperand::new(OperandData::None),
                        MOperand::input(OperandData::None),
                        MOperand::input(OperandData::None),
                        MOperand::new(OperandData::None),
                        MOperand::input(inst.into()),
                    ],
                })]);
            return;
        }
        _ => {}
    }

    match (slot, const_int) {
        (Some(slot), Some(ConstantInt::Int32(imm))) => {
            ctx.inst_seq.append(&mut vec![MachInstruction {
                id: None,
                data: InstructionData {
                    opcode: Opcode::MOVmi32,
                    operands: vec![
                        MOperand::new(OperandData::MemStart),
                        MOperand::output(OperandData::Slot(slot)),
                        MOperand::new(OperandData::None),
                        MOperand::input(OperandData::None),
                        MOperand::input(OperandData::None),
                        MOperand::new(OperandData::None),
                        MOperand::input(imm.into()),
                    ],
                },
            }]);
            return;
        }
        _ => todo!(),
    }
}

fn lower_store_gep(
    ctx: &mut LoweringContext<X86_64>,
    tys: &[TypeId],
    args: &[ValueId],
    _align: u32,
    gep_id: InstructionId,
) {
    let gep = &ctx.ir_data.instructions[gep_id];

    // The simplest pattern
    if let &[base_ptr, idx0, idx1] = gep.operand.args() {
        // let base_ty = gep.operand.types()[0];
        let base_ptr = ctx.inst_id_to_slot_id[ctx.ir_data.values[base_ptr].as_inst()];

        let idx0_ty = gep.operand.types()[2];
        assert_eq!(*ctx.types.get(idx0_ty), Type::Int(64));
        let idx0 = match ctx.ir_data.values[idx0] {
            Value::Constant(ConstantData::Int(ConstantInt::Int64(idx))) => idx,
            _ => todo!(),
        };

        let idx1_ty = gep.operand.types()[3];
        assert_eq!(*ctx.types.get(idx1_ty), Type::Int(64));
        let mut idx1_const = None;
        let mut idx1_vreg = None;
        match ctx.ir_data.values[idx1] {
            Value::Constant(ConstantData::Int(ConstantInt::Int64(idx))) => idx1_const = Some(idx),
            Value::Instruction(id) => {
                idx1_vreg = Some(get_or_generate_inst_output(ctx, idx1_ty, id))
            }
            _ => todo!(),
        };

        let mem_op = if let Some(idx1) = idx1_const {
            // idx0 * (size of base_ty) + idx1 * (size of inner of base_ty)
            let base_ty = gep.operand.types()[0];
            let offset = idx0 * X86_64::type_size(ctx.types, base_ty) as i64
                + idx1
                    * X86_64::type_size(ctx.types, ctx.types.get_element(base_ty).unwrap()) as i64;
            debug!(offset);

            vec![
                MOperand::new(OperandData::MemStart),
                MOperand::new(OperandData::Slot(base_ptr)),
                MOperand::new(OperandData::Int32(offset as i32)),
                MOperand::input(OperandData::None),
                MOperand::input(OperandData::None),
                MOperand::new(OperandData::None),
            ]
        } else if let Some(idx1) = idx1_vreg {
            let base_ty = gep.operand.types()[0];
            let offset = idx0 * X86_64::type_size(ctx.types, base_ty) as i64;
            debug!(offset);

            assert!(
                X86_64::type_size(ctx.types, ctx.types.get_element(base_ty).unwrap()) as i8 == 4
            );

            vec![
                MOperand::new(OperandData::MemStart),
                MOperand::new(OperandData::Slot(base_ptr)),
                MOperand::new(OperandData::Int32(offset as i32)),
                MOperand::input(OperandData::None),
                MOperand::input(OperandData::VReg(idx1)),
                MOperand::new(OperandData::Int32(X86_64::type_size(
                    ctx.types,
                    ctx.types.get_element(base_ty).unwrap(),
                ) as i32)),
            ]
        } else {
            panic!()
        };

        match ctx.ir_data.value_ref(args[0]) {
            Value::Constant(ConstantData::Int(ConstantInt::Int32(int))) => {
                ctx.inst_seq
                    .append(&mut vec![MachInstruction::new(InstructionData {
                        opcode: Opcode::MOVmi32,
                        operands: mem_op
                            .into_iter()
                            .chain(vec![MOperand::input(int.into())].into_iter())
                            .collect(),
                    })]);
            }
            Value::Instruction(id) => {
                let src = get_or_generate_inst_output(ctx, tys[0], *id);
                ctx.inst_seq
                    .append(&mut vec![MachInstruction::new(InstructionData {
                        opcode: Opcode::MOVmr32,
                        operands: mem_op
                            .into_iter()
                            .chain(vec![MOperand::input(src.into())].into_iter())
                            .collect(),
                    })]);
            }
            _ => todo!(),
        }

        return;
    }

    todo!()
}

fn lower_add(ctx: &mut LoweringContext<X86_64>, id: InstructionId, ty: TypeId, args: &[ValueId]) {
    let lhs = val_to_vreg(ctx, ty, args[0]);
    let output = new_empty_inst_output(ctx, ty, id);

    let insert_move = |ctx: &mut LoweringContext<X86_64>| {
        ctx.inst_seq.push(MachInstruction {
            id: None,
            data: InstructionData {
                opcode: Opcode::MOVrr32,
                operands: vec![MOperand::output(output.into()), MOperand::input(lhs.into())],
            },
        })
    };

    let rhs = val_to_operand_data(ctx, ty, args[1]);

    let data = match rhs {
        OperandData::Int32(rhs) => {
            insert_move(ctx);
            InstructionData {
                opcode: Opcode::ADDri32,
                operands: vec![
                    MOperand::input_output(output.into()),
                    MOperand::new(rhs.into()),
                ],
            }
        }
        OperandData::VReg(rhs) => {
            insert_move(ctx);
            InstructionData {
                opcode: Opcode::ADDrr32,
                operands: vec![
                    MOperand::input_output(output.into()),
                    MOperand::input(rhs.into()),
                ],
            }
        }
        _ => todo!(),
    };

    ctx.inst_seq.push(MachInstruction { id: None, data });
}

fn lower_sext(
    ctx: &mut LoweringContext<X86_64>,
    id: InstructionId,
    tys: &[TypeId; 2],
    arg: ValueId,
) {
    let from = tys[0];
    let to = tys[1];
    assert_eq!(*ctx.types.get(from), Type::Int(32));
    assert_eq!(*ctx.types.get(to), Type::Int(64));

    let val = match ctx.ir_data.values[arg] {
        Value::Instruction(id) => get_or_generate_inst_output(ctx, from, id),
        _ => todo!(),
    };

    let output = new_empty_inst_output(ctx, to, id);

    ctx.inst_seq.push(MachInstruction {
        id: None,
        data: InstructionData {
            opcode: Opcode::MOVSXDr64r32,
            operands: vec![MOperand::output(output.into()), MOperand::input(val.into())],
        },
    });
}

fn lower_br(ctx: &mut LoweringContext<X86_64>, block: BasicBlockId) {
    ctx.inst_seq.push(MachInstruction {
        id: None,
        data: InstructionData {
            opcode: Opcode::JMP,
            operands: vec![MOperand::new(OperandData::Block(ctx.block_map[&block]))],
        },
    })
}

fn lower_condbr(ctx: &mut LoweringContext<X86_64>, arg: ValueId, blocks: [BasicBlockId; 2]) {
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
        let lhs = val_to_vreg(ctx, *ty, args[0]);
        let rhs = ctx.ir_data.value_ref(args[1]);
        match rhs {
            Value::Constant(ConstantData::Int(ConstantInt::Int32(rhs))) => {
                ctx.inst_seq.push(MachInstruction::new(InstructionData {
                    opcode: Opcode::CMPri32,
                    operands: vec![MOperand::input(lhs.into()), MOperand::new(rhs.into())],
                }));
            }
            _ => todo!(),
        }

        ctx.inst_seq.push(MachInstruction::new(InstructionData {
            opcode: match cond {
                ICmpCond::Eq => Opcode::JE,
                ICmpCond::Ne => Opcode::JNE,
                ICmpCond::Sle => Opcode::JLE,
                ICmpCond::Slt => Opcode::JL,
                ICmpCond::Sge => Opcode::JGE,
                ICmpCond::Sgt => Opcode::JG,
                _ => todo!(),
            },
            operands: vec![MOperand::new(OperandData::Block(ctx.block_map[&blocks[0]]))],
        }));
        ctx.inst_seq.push(MachInstruction::new(InstructionData {
            opcode: Opcode::JMP,
            operands: vec![MOperand::new(OperandData::Block(ctx.block_map[&blocks[1]]))],
        }));
        return;
    }

    todo!()
}

fn lower_call(
    ctx: &mut LoweringContext<X86_64>,
    id: InstructionId,
    tys: &[TypeId],
    args: &[ValueId],
) {
    let output = new_empty_inst_output(ctx, tys[0], id);

    let gpru = RegInfo::arg_reg_list(&ctx.call_conv);
    let mut gpr_used = 0;
    for (&arg, &ty) in args[1..].iter().zip(tys[1..].iter()) {
        let arg = val_to_operand_data(ctx, ty, arg);
        let r = gpru[gpr_used].apply(&RegClass::for_type(ctx.types, ty));
        gpr_used += 1;
        ctx.inst_seq.push(MachInstruction {
            id: None,
            data: InstructionData {
                opcode: match &arg {
                    OperandData::Int32(_) => Opcode::MOVri32,
                    OperandData::VReg(_) | OperandData::Reg(_) => Opcode::MOVrr32,
                    _ => todo!(),
                },
                operands: vec![MOperand::output(r.into()), MOperand::input(arg.into())],
            },
        });
    }

    let name = match &ctx.ir_data.values[args[0]] {
        Value::Constant(ConstantData::GlobalRef(Name::Name(name))) => name.clone(),
        _ => todo!(),
    };
    let result_reg: Reg = GR32::EAX.into(); // TODO: do not hard code
    ctx.inst_seq.push(MachInstruction::new(InstructionData {
        opcode: Opcode::CALL,
        operands: vec![
            MOperand::implicit_output(result_reg.into()),
            MOperand::new(OperandData::Label(name)),
        ],
    }));
    ctx.inst_seq.push(MachInstruction::new(InstructionData {
        opcode: Opcode::MOVrr32,
        operands: vec![
            MOperand::output(output.into()),
            MOperand::input(result_reg.into()),
        ],
    }));
}

fn lower_return(ctx: &mut LoweringContext<X86_64>, ty: TypeId, value: ValueId) {
    let vreg = val_to_vreg(ctx, ty, value);
    assert!(*ctx.types.get(ty) == Type::Int(32));
    ctx.inst_seq.push(MachInstruction {
        id: None,
        data: InstructionData {
            opcode: Opcode::MOVrr32,
            operands: vec![
                MOperand::output(OperandData::Reg(GR32::EAX.into())),
                MOperand::input(vreg.into()),
            ],
        },
    });
    ctx.inst_seq.push(MachInstruction {
        id: None,
        data: InstructionData {
            opcode: Opcode::RET,
            operands: vec![],
        },
    });
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
) -> VReg {
    if let Some(vreg) = ctx.inst_id_to_vreg.get(&id) {
        return *vreg;
    }

    if ctx.ir_data.inst_ref(id).parent != ctx.cur_block {
        // The instruction indexed as `id` must be placed in another basic block
        let v = ctx.vregs.add_vreg_data(ty);
        ctx.inst_id_to_vreg.insert(id, v);
        return v;
    }

    // TODO: What about instruction scheduling?
    lower(ctx, ctx.ir_data.inst_ref(id));
    get_or_generate_inst_output(ctx, ty, id)
}

fn new_empty_inst_output(ctx: &mut LoweringContext<X86_64>, ty: TypeId, id: InstructionId) -> VReg {
    if let Some(vreg) = ctx.inst_id_to_vreg.get(&id) {
        return *vreg;
    }
    let vreg = ctx.vregs.add_vreg_data(ty);
    ctx.inst_id_to_vreg.insert(id, vreg);
    vreg
}

fn val_to_operand_data(ctx: &mut LoweringContext<X86_64>, ty: TypeId, val: ValueId) -> OperandData {
    match ctx.ir_data.values[val] {
        Value::Instruction(id) => get_or_generate_inst_output(ctx, ty, id).into(),
        Value::Argument(idx) => ctx.arg_idx_to_vreg[&idx].into(),
        Value::Constant(ConstantData::Int(ConstantInt::Int32(i))) => OperandData::Int32(i),
        _ => todo!(),
    }
}

fn val_to_vreg(ctx: &mut LoweringContext<X86_64>, ty: TypeId, val: ValueId) -> VReg {
    match val_to_operand_data(ctx, ty, val) {
        OperandData::Int32(i) => {
            let output = ctx.vregs.add_vreg_data(ty);
            ctx.inst_seq.push(MachInstruction {
                id: None,
                data: InstructionData {
                    opcode: Opcode::MOVri32,
                    operands: vec![MOperand::output(output.into()), MOperand::new(i.into())],
                },
            });
            output
        }
        OperandData::VReg(vr) => vr,
        _ => todo!(),
    }
}
