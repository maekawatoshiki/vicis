use crate::codegen::{
    calling_conv::CallingConv,
    function::instruction::Instruction as MachInstruction,
    lower::pattern::{Lower as LowerTrait, LoweringContext},
    register::VReg,
    target::x86_64::{
        instruction::{InstructionData, MemoryOperand, Opcode, Operand as MOperand, OperandData},
        register::{RegClass, GR32},
        X86_64,
    },
};
use crate::ir::{
    function::instruction::{Instruction as IrInstruction, InstructionId, Operand},
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

impl<CC: CallingConv<RegClass>> LowerTrait<X86_64<CC>> for Lower {
    fn lower(&self, ctx: &mut LoweringContext<X86_64<CC>>, inst: &IrInstruction) {
        lower(ctx, inst)
    }
}

fn lower<CC: CallingConv<RegClass>>(ctx: &mut LoweringContext<X86_64<CC>>, inst: &IrInstruction) {
    match inst.operand {
        Operand::Alloca {
            ref tys,
            ref num_elements,
            align,
        } => lower_alloca(ctx, inst.id.unwrap(), tys, num_elements, align),
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
        Operand::Ret { val: None, .. } => todo!(),
        Operand::Ret { val: Some(val), ty } => lower_return(ctx, ty, val),
        _ => todo!(),
    }
}

fn lower_alloca<CC: CallingConv<RegClass>>(
    ctx: &mut LoweringContext<X86_64<CC>>,
    id: InstructionId,
    tys: &[TypeId],
    _num_elements: &ConstantData,
    _align: u32,
) {
    let slot_id = ctx.slots.add_slot(tys[0]);
    ctx.inst_id_to_slot_id.insert(id, slot_id);
}

fn lower_load<CC: CallingConv<RegClass>>(
    ctx: &mut LoweringContext<X86_64<CC>>,
    id: InstructionId,
    tys: &[TypeId],
    addr: ValueId,
    _align: u32,
) {
    let mut slot = None;

    match ctx.ir_data.value_ref(addr) {
        Value::Instruction(id) => {
            if let Some(slot_id) = ctx.inst_id_to_slot_id.get(id) {
                slot = Some(slot_id);
            }
        }
        _ => todo!(),
    }

    if let Some(slot) = slot {
        if matches!(&*ctx.types.get(tys[0]), Type::Int(32)) {
            let vreg = ctx.vregs.add_vreg_data(tys[0]);
            ctx.inst_id_to_vreg.insert(id, vreg);
            ctx.inst_seq.push(MachInstruction {
                id: None,
                data: InstructionData {
                    opcode: Opcode::MOVrm32,
                    operands: vec![
                        MOperand::output(OperandData::VReg(vreg)),
                        MOperand::input(OperandData::Mem(MemoryOperand::Slot(*slot))),
                    ],
                },
            });
            return;
        }
    }

    todo!()
}

fn lower_store<CC: CallingConv<RegClass>>(
    ctx: &mut LoweringContext<X86_64<CC>>,
    _tys: &[TypeId],
    args: &[ValueId],
    _align: u32,
) {
    let mut slot = None;

    match ctx.ir_data.value_ref(args[1]) {
        Value::Instruction(id) => {
            if let Some(slot_id) = ctx.inst_id_to_slot_id.get(id) {
                slot = Some(slot_id);
            }
        }
        _ => todo!(),
    }

    let mut const_int = None;

    match ctx.ir_data.value_ref(args[0]) {
        Value::Constant(ConstantData::Int(int)) => const_int = Some(*int),
        _ => {}
    }

    match (slot, const_int) {
        (Some(slot), Some(ConstantInt::Int32(imm))) => {
            ctx.inst_seq.append(&mut vec![MachInstruction {
                id: None,
                data: InstructionData {
                    opcode: Opcode::MOVmi32,
                    operands: vec![
                        MOperand::output(OperandData::Mem(MemoryOperand::Slot(*slot))),
                        MOperand::input(OperandData::Int32(imm)),
                    ],
                },
            }]);
            return;
        }
        _ => todo!(),
    }
}

fn lower_add<CC: CallingConv<RegClass>>(
    ctx: &mut LoweringContext<X86_64<CC>>,
    id: InstructionId,
    ty: TypeId,
    args: &[ValueId],
) {
    let lhs;
    let rhs = ctx.ir_data.value_ref(args[1]);
    let new;

    if let Value::Instruction(l) = ctx.ir_data.value_ref(args[0]) {
        lhs = get_inst_output(ctx, *l);
        new = ctx.vregs.add_vreg_data(ty);
        ctx.inst_id_to_vreg.insert(id, new);
    } else {
        panic!();
    };

    let insert_move = |ctx: &mut LoweringContext<X86_64<CC>>| {
        ctx.inst_seq.push(MachInstruction {
            id: None,
            data: InstructionData {
                opcode: Opcode::MOVrr32,
                operands: vec![
                    MOperand::output(OperandData::VReg(new)),
                    MOperand::input(OperandData::VReg(lhs)),
                ],
            },
        })
    };

    if let Value::Instruction(rhs) = rhs {
        let rhs = get_inst_output(ctx, *rhs);
        insert_move(ctx);
        ctx.inst_seq.push(MachInstruction {
            id: None,
            data: InstructionData {
                opcode: Opcode::ADDrr32,
                operands: vec![
                    MOperand::input_output(OperandData::VReg(new)),
                    MOperand::output(OperandData::VReg(rhs)),
                ],
            },
        });
        return;
    }

    if let Value::Constant(ConstantData::Int(ConstantInt::Int32(rhs))) = rhs {
        insert_move(ctx);
        ctx.inst_seq.push(MachInstruction {
            id: None,
            data: InstructionData {
                opcode: Opcode::ADDrr32,
                operands: vec![
                    MOperand::input_output(OperandData::VReg(new)),
                    MOperand::output(OperandData::Int32(*rhs)),
                ],
            },
        });
        return;
    }

    todo!()
}

fn lower_return<CC: CallingConv<RegClass>>(
    ctx: &mut LoweringContext<X86_64<CC>>,
    _ty: TypeId,
    value: ValueId,
) {
    let value = ctx.ir_data.value_ref(value);
    match value {
        Value::Constant(ConstantData::Int(ConstantInt::Int32(i))) => {
            ctx.inst_seq.push(MachInstruction {
                id: None,
                data: InstructionData {
                    opcode: Opcode::MOVri32,
                    operands: vec![
                        MOperand::output(OperandData::Reg(GR32::EAX.into())),
                        MOperand::input(OperandData::Int32(*i)),
                    ],
                },
            });
        }
        Value::Instruction(id) => {
            let vreg = get_inst_output(ctx, *id);
            ctx.inst_seq.push(MachInstruction {
                id: None,
                data: InstructionData {
                    opcode: Opcode::MOVrr32,
                    operands: vec![
                        MOperand::output(OperandData::Reg(GR32::EAX.into())),
                        MOperand::input(OperandData::VReg(vreg)),
                    ],
                },
            });
        }
        _ => todo!(),
    }
    ctx.inst_seq.push(MachInstruction {
        id: None,
        data: InstructionData {
            opcode: Opcode::RET,
            operands: vec![],
        },
    });
}

fn get_inst_output<CC: CallingConv<RegClass>>(
    ctx: &mut LoweringContext<X86_64<CC>>,
    id: InstructionId,
) -> VReg {
    if let Some(vreg) = ctx.inst_id_to_vreg.get(&id) {
        return *vreg;
    }

    lower(ctx, ctx.ir_data.inst_ref(id));
    get_inst_output(ctx, id)
}
