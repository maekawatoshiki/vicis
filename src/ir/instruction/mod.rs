pub mod parser;

pub use parser::parse;

use super::{
    basic_block::BasicBlockId, function::Data, module::name::Name, types::TypeId, types::Types,
    value::ValueId,
};
use id_arena::Id;
use rustc_hash::FxHashSet;
use std::{fmt, slice};

pub type InstructionId = Id<Instruction>;

pub struct Instruction {
    pub opcode: Opcode,
    pub operand: Operand,
    pub dest: Option<Name>,
    pub id: Option<InstructionId>,
    pub parent: BasicBlockId,
    pub users: FxHashSet<InstructionId>,
    // pub result_ty: Option<TypeId>
}

#[derive(Clone, Copy)]
pub enum Opcode {
    Alloca,
    Load,
    Store,
    Add,
    Sub,
    Mul,
    ICmp,
    Zext,
    Call,
    Br,
    CondBr,
    Ret,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum ICmpCond {
    Eq,
    Ne,
    Ugt,
    Uge,
    Ult,
    Ule,
    Sgt,
    Sge,
    Slt,
    Sle,
}

#[derive(Clone)]
pub enum Operand {
    Alloca {
        tys: [TypeId; 2],
        num_elements: ValueId,
        align: u32,
    },
    Load {
        tys: [TypeId; 2],
        addr: ValueId,
        align: u32,
    },
    IntBinary {
        ty: TypeId,
        nsw: bool,
        nuw: bool,
        args: [ValueId; 2],
    },
    // IntDiv { .. }
    Store {
        tys: [TypeId; 2],
        args: [ValueId; 2],
        align: u32,
    },
    ICmp {
        ty: TypeId,
        args: [ValueId; 2],
        cond: ICmpCond,
    },
    Cast {
        tys: [TypeId; 2], // from, to
        arg: ValueId,
    },
    Call {
        args: Vec<ValueId>, // args[0] = callee, args[1..] = arguments
        tys: Vec<TypeId>,   // tys[0] = callee's result type, args[1..] = argument types
    },
    Br {
        block: BasicBlockId,
    },
    CondBr {
        arg: ValueId,
        blocks: [BasicBlockId; 2], // iftrue, iffalse
    },
    Ret {
        ty: TypeId,
        val: Option<ValueId>,
    },
    Invalid,
}

impl Instruction {
    pub fn with_operand(mut self, operand: Operand) -> Self {
        self.operand = operand;
        self
    }

    pub fn with_dest(mut self, dest: Name) -> Self {
        self.dest = Some(dest);
        self
    }

    pub fn to_string(&self, data: &Data, types: &Types) -> String {
        match &self.operand {
            Operand::Alloca {
                tys,
                num_elements,
                align,
            } => {
                // TODO: %I{index} or %{self.dest}
                format!(
                    "%I{} = alloca {}, {} {}, align {}",
                    self.id.unwrap().index(),
                    types.to_string(tys[0]),
                    types.to_string(tys[1]),
                    data.value_ref(*num_elements).to_string(data, types),
                    align
                )
            }
            Operand::Load { tys, addr, align } => {
                format!(
                    "%I{} = load {}, {} {}, align {}",
                    self.id.unwrap().index(),
                    types.to_string(tys[0]),
                    types.to_string(tys[1]),
                    data.value_ref(*addr).to_string(data, types),
                    align
                )
            }
            Operand::Store { tys, args, align } => {
                format!(
                    "store {} {}, {} {}, align {}",
                    types.to_string(tys[0]),
                    data.value_ref(args[0]).to_string(data, types),
                    types.to_string(tys[1]),
                    data.value_ref(args[1]).to_string(data, types),
                    align
                )
            }
            Operand::IntBinary { ty, nuw, nsw, args } => {
                format!(
                    "%I{} = {:?}{}{} {} {}, {}",
                    self.id.unwrap().index(),
                    self.opcode,
                    if *nuw { " nuw" } else { "" },
                    if *nsw { " nsw" } else { "" },
                    types.to_string(*ty),
                    data.value_ref(args[0]).to_string(data, types),
                    data.value_ref(args[1]).to_string(data, types),
                )
            }
            Operand::ICmp { ty, args, cond } => {
                format!(
                    "%I{} = icmp {:?} {} {}, {}",
                    self.id.unwrap().index(),
                    cond,
                    types.to_string(*ty),
                    data.value_ref(args[0]).to_string(data, types),
                    data.value_ref(args[1]).to_string(data, types)
                )
            }
            Operand::Cast { tys, arg } => {
                format!(
                    "%I{} = {:?} {} {} to {}",
                    self.id.unwrap().index(),
                    self.opcode,
                    types.to_string(tys[0]),
                    data.value_ref(*arg).to_string(data, types),
                    types.to_string(tys[1]),
                )
            }
            Operand::Call { tys, args } => {
                format!(
                    "%I{} = call {} {}({})",
                    self.id.unwrap().index(),
                    types.to_string(tys[0]),
                    data.value_ref(args[0]).to_string(data, types),
                    tys[1..]
                        .iter()
                        .zip(args[1..].iter())
                        .into_iter()
                        .fold("".to_string(), |acc, (t, a)| {
                            format!(
                                "{}{} {}, ",
                                acc,
                                types.to_string(*t),
                                data.value_ref(*a).to_string(data, types),
                            )
                        })
                        .trim_end_matches(", ")
                )
            }
            Operand::Br { block } => {
                format!("br label %B{}", block.index())
            }
            Operand::CondBr { arg, blocks } => {
                format!(
                    "br i1 {}, label %B{}, label %B{}",
                    data.value_ref(*arg).to_string(data, types),
                    blocks[0].index(),
                    blocks[1].index()
                )
            }
            Operand::Ret { val: None, .. } => format!("ret void"),
            Operand::Ret { val: Some(val), ty } => {
                format!(
                    "ret {} {}",
                    types.to_string(*ty),
                    data.value_ref(*val).to_string(data, types)
                )
            }
            Operand::Invalid => panic!(),
        }
    }
}

impl Opcode {
    pub fn with_block(self, parent: BasicBlockId) -> Instruction {
        Instruction {
            opcode: self,
            operand: Operand::Invalid,
            dest: None,
            id: None,
            parent,
            users: FxHashSet::default(),
        }
    }

    pub fn is_terminator(&self) -> bool {
        matches!(self, Self::Ret | Self::Br | Self::CondBr)
    }
}

impl Operand {
    pub fn args(&self) -> &[ValueId] {
        match self {
            Self::Alloca { num_elements, .. } => slice::from_ref(num_elements),
            Self::Ret { val, .. } if val.is_none() => &[],
            Self::Ret { val, .. } => slice::from_ref(val.as_ref().unwrap()),
            Self::Load { addr, .. } => slice::from_ref(addr),
            Self::Store { args, .. } => args,
            Self::IntBinary { args, .. } => args,
            Self::ICmp { args, .. } => args,
            Self::Cast { arg, .. } => slice::from_ref(arg),
            Self::Call { args, .. } => args.as_slice(),
            Self::Br { .. } => &[],
            Self::CondBr { arg, .. } => slice::from_ref(arg),
            Self::Invalid => &[],
        }
    }

    pub fn types(&self) -> &[TypeId] {
        match self {
            Self::Alloca { tys, .. } => tys,
            Self::Ret { ty, .. } => slice::from_ref(ty),
            Self::Load { tys, .. } => tys,
            Self::Store { .. } => &[],
            Self::IntBinary { ty, .. } => slice::from_ref(ty),
            Self::ICmp { ty, .. } => slice::from_ref(ty),
            Self::Cast { tys, .. } => tys,
            Self::Call { tys, .. } => tys.as_slice(),
            Self::Br { .. } => &[],
            Self::CondBr { .. } => &[],
            Self::Invalid => &[],
        }
    }

    pub fn blocks(&self) -> &[BasicBlockId] {
        match self {
            Self::Br { block } => slice::from_ref(block),
            Self::CondBr { blocks, .. } => blocks,
            _ => &[],
        }
    }
}

impl fmt::Debug for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Opcode::Alloca => "alloca",
                Opcode::Load => "load",
                Opcode::Store => "store",
                Opcode::Add => "add",
                Opcode::Sub => "sub",
                Opcode::Mul => "mul",
                Opcode::ICmp => "icmp",
                Opcode::Zext => "zext",
                Opcode::Call => "call",
                Opcode::Br | Opcode::CondBr => "br",
                Opcode::Ret => "ret",
            }
        )
    }
}

impl fmt::Debug for ICmpCond {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Eq => "eq",
                Self::Ne => "ne",
                Self::Ugt => "ugt",
                Self::Uge => "uge",
                Self::Ult => "ult",
                Self::Ule => "ule",
                Self::Sgt => "sgt",
                Self::Sge => "sge",
                Self::Slt => "slt",
                Self::Sle => "sle",
            }
        )
    }
}
