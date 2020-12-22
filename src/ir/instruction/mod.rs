pub mod parser;

pub use parser::parse;

use super::{
    basic_block::BasicBlockId, function::Data, module::name::Name, types::TypeId, types::Types,
    value::ValueId,
};
use id_arena::Id;
use std::slice;

pub type InstructionId = Id<Instruction>;

pub struct Instruction {
    pub opcode: Opcode,
    pub operand: Operand,
    pub dest: Option<Name>,
    pub id: Option<InstructionId>,
    pub parent: BasicBlockId,
    // pub result_ty: Option<TypeIdjj
}

pub enum Opcode {
    Alloca,
    Load,
    Store,
    Add,
    Ret,
}

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
    Add {
        ty: TypeId,
        nsw: bool,
        nuw: bool,
        args: [ValueId; 2],
    },
    Store {
        tys: [TypeId; 2],
        args: [ValueId; 2],
        align: u32,
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
                // TODO: %id_{index} or %{self.dest}
                format!(
                    "%id{} = alloca {}, {} {}, align {}",
                    self.id.unwrap().index(),
                    types.to_string(tys[0]),
                    types.to_string(tys[1]),
                    data.value_ref(*num_elements).to_string(data, types),
                    align
                )
            }
            Operand::Load { tys, addr, align } => {
                format!(
                    "%id{} = load {}, {} {}, align {}",
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
            Operand::Add { ty, nuw, nsw, args } => {
                format!(
                    "%id{} = add{}{} {} {}, {}",
                    self.id.unwrap().index(),
                    if *nuw { " nuw" } else { "" },
                    if *nsw { " nsw" } else { "" },
                    types.to_string(*ty),
                    data.value_ref(args[0]).to_string(data, types),
                    data.value_ref(args[1]).to_string(data, types),
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
        }
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
            Self::Add { args, .. } => args,
            Self::Invalid => &[],
        }
    }

    pub fn types(&self) -> &[TypeId] {
        match self {
            Self::Alloca { tys, .. } => tys,
            Self::Ret { ty, .. } => slice::from_ref(ty),
            Self::Load { tys, .. } => tys,
            Self::Store { .. } => &[],
            Self::Add { ty, .. } => slice::from_ref(ty),
            Self::Invalid => &[],
        }
    }
}
