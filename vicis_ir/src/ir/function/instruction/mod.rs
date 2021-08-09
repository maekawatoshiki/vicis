pub mod builder;
pub mod parser;

pub use parser::parse;
use rustc_hash::FxHashMap;

use crate::ir::{
    function::{basic_block::BasicBlockId, data::Data, param_attrs::ParameterAttribute},
    module::{attributes::Attribute, metadata::Metadata, name::Name},
    types::TypeId,
    value::{ConstantData, ConstantInt, Value, ValueId},
};
use id_arena::Id;
use std::{fmt, slice};

pub type InstructionId = Id<Instruction>;

pub struct Instruction {
    pub opcode: Opcode,
    pub operand: Operand,
    pub dest: Option<Name>,
    pub id: Option<InstructionId>,
    pub parent: BasicBlockId,
    pub metadata: FxHashMap<String, Metadata>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Opcode {
    Alloca,
    Phi,
    Load,
    Store,
    InsertValue,
    ExtractValue,
    Add,
    Sub,
    Mul,
    SDiv,
    SRem,
    And,
    LShr,
    ICmp,
    Sext,
    Zext,
    Bitcast,
    Trunc,
    IntToPtr,
    GetElementPtr,
    Call,
    Invoke,
    LandingPad,
    Resume,
    Br,
    CondBr,
    Ret,
    Invalid,
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

#[derive(Debug, Clone)]
pub struct Alloca {
    pub tys: [TypeId; 2],
    pub num_elements: ConstantData,
    pub align: u32,
}

#[derive(Debug, Clone)]
pub struct Phi {
    pub ty: TypeId,
    pub args: Vec<ValueId>,
    pub blocks: Vec<BasicBlockId>,
}

#[derive(Debug, Clone)]
pub struct Load {
    pub tys: [TypeId; 2],
    pub addr: ValueId,
    pub align: u32,
}

#[derive(Debug, Clone)]
pub struct IntBinary {
    pub ty: TypeId,
    pub nsw: bool,
    pub nuw: bool,
    pub exact: bool,
    pub args: [ValueId; 2],
}

#[derive(Debug, Clone)]
pub struct Store {
    pub tys: [TypeId; 2],
    pub args: [ValueId; 2],
    pub align: u32,
}

#[derive(Debug, Clone)]
pub struct InsertValue {
    pub tys: [TypeId; 2],
    pub args: Vec<ValueId>,
}

#[derive(Debug, Clone)]
pub struct ExtractValue {
    pub ty: TypeId,
    pub args: Vec<ValueId>,
}

#[derive(Debug, Clone)]
pub struct ICmp {
    pub ty: TypeId,
    pub args: [ValueId; 2],
    pub cond: ICmpCond,
}

#[derive(Debug, Clone)]
pub struct Cast {
    pub tys: [TypeId; 2], // from, to
    pub arg: ValueId,
}

#[derive(Debug, Clone)]
pub struct GetElementPtr {
    pub inbounds: bool,
    pub tys: Vec<TypeId>,
    pub args: Vec<ValueId>,
}

#[derive(Debug, Clone)]
pub struct Call {
    pub args: Vec<ValueId>, // args[0] = callee, args[1..] = arguments
    pub tys: Vec<TypeId>,   // tys[0] = callee's result type, args[1..] = argument types
    pub param_attrs: Vec<Vec<ParameterAttribute>>, // param_attrs[0] = attrs of args[1]
    pub ret_attrs: Vec<ParameterAttribute>,
    pub func_attrs: Vec<Attribute>,
}

#[derive(Debug, Clone)]
pub struct Invoke {
    pub args: Vec<ValueId>, // args[0] = callee, args[1..] = arguments
    pub tys: Vec<TypeId>,   // tys[0] = callee's result type, args[1..] = argument types
    pub param_attrs: Vec<Vec<ParameterAttribute>>, // param_attrs[0] = attrs of args[1]
    pub ret_attrs: Vec<ParameterAttribute>,
    pub func_attrs: Vec<Attribute>,
    pub blocks: Vec<BasicBlockId>,
}

#[derive(Debug, Clone)]
pub struct LandingPad {
    pub ty: TypeId,
}

#[derive(Debug, Clone)]
pub struct Resume {
    pub ty: TypeId,
    pub arg: ValueId,
}

#[derive(Debug, Clone)]
pub struct Br {
    pub block: BasicBlockId,
}

#[derive(Debug, Clone)]
pub struct CondBr {
    pub arg: ValueId,
    pub blocks: [BasicBlockId; 2], // iftrue, iffalse
}

#[derive(Debug, Clone)]
pub struct Ret {
    pub ty: TypeId,
    pub val: Option<ValueId>,
}

#[derive(Clone)]
pub enum Operand {
    Alloca(Alloca),
    Phi(Phi),
    Load(Load),
    IntBinary(IntBinary),
    Store(Store),
    InsertValue(InsertValue),
    ExtractValue(ExtractValue),
    ICmp(ICmp),
    Cast(Cast),
    GetElementPtr(GetElementPtr),
    Call(Call),
    Invoke(Invoke),
    LandingPad(LandingPad),
    Resume(Resume),
    Br(Br),
    CondBr(CondBr),
    Ret(Ret),
    Invalid,
}

impl Instruction {
    pub fn replace(&mut self, other: Self) {
        assert_eq!(self.opcode, Opcode::Invalid);
        self.opcode = other.opcode;
        self.operand = other.operand;
        self.dest = other.dest;
        self.parent = other.parent;
    }

    pub fn with_operand(mut self, operand: Operand) -> Self {
        self.operand = operand;
        self
    }

    pub fn with_dest(mut self, dest: Name) -> Self {
        self.dest = Some(dest);
        self
    }

    pub fn with_metadata(mut self, metadata: FxHashMap<String, Metadata>) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn fold_consts(&self, data: &Data) -> Option<ConstantData> {
        match self.operand {
            Operand::IntBinary(ref i) => {
                let args: Vec<&Value> = i.args.iter().map(|&id| data.value_ref(id)).collect();
                match args.as_slice() {
                    [Value::Constant(ConstantData::Int(ConstantInt::Int32(x))), Value::Constant(ConstantData::Int(ConstantInt::Int32(y)))] => {
                        match self.opcode {
                            Opcode::Add => Some(ConstantData::Int(ConstantInt::Int32(x + y))),
                            Opcode::Sub => Some(ConstantData::Int(ConstantInt::Int32(x - y))),
                            Opcode::Mul => Some(ConstantData::Int(ConstantInt::Int32(x * y))),
                            Opcode::SRem => Some(ConstantData::Int(ConstantInt::Int32(x % y))),
                            _ => None,
                        }
                    }
                    _ => None,
                }
            }
            Operand::ICmp(ref i) => {
                let args: Vec<&Value> = i.args.iter().map(|&id| data.value_ref(id)).collect();
                match args.as_slice() {
                    [Value::Constant(ConstantData::Int(ConstantInt::Int32(x))), Value::Constant(ConstantData::Int(ConstantInt::Int32(y)))] => {
                        match i.cond {
                            ICmpCond::Eq => Some(ConstantData::Int(ConstantInt::Int1(x == y))),
                            ICmpCond::Ne => Some(ConstantData::Int(ConstantInt::Int1(x != y))),
                            ICmpCond::Ugt => Some(ConstantData::Int(ConstantInt::Int1(x > y))),
                            ICmpCond::Uge => Some(ConstantData::Int(ConstantInt::Int1(x >= y))),
                            ICmpCond::Ult => Some(ConstantData::Int(ConstantInt::Int1(x < y))),
                            ICmpCond::Ule => Some(ConstantData::Int(ConstantInt::Int1(x <= y))),
                            ICmpCond::Sgt => Some(ConstantData::Int(ConstantInt::Int1(x > y))),
                            ICmpCond::Sge => Some(ConstantData::Int(ConstantInt::Int1(x >= y))),
                            ICmpCond::Slt => Some(ConstantData::Int(ConstantInt::Int1(x < y))),
                            ICmpCond::Sle => Some(ConstantData::Int(ConstantInt::Int1(x <= y))),
                        }
                    }
                    _ => None,
                }
            }
            Operand::Cast(_) => todo!(),
            _ => None,
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
            metadata: FxHashMap::default(), // users: FxHashSet::default(),
        }
    }

    pub fn is_terminator(&self) -> bool {
        matches!(
            self,
            Self::Ret | Self::Br | Self::CondBr | Self::Invoke | Self::Resume
        )
    }

    pub fn is_load(&self) -> bool {
        self == &Self::Load
    }

    pub fn is_store(&self) -> bool {
        self == &Self::Store
    }

    pub fn is_alloca(&self) -> bool {
        self == &Self::Alloca
    }

    pub fn is_phi(&self) -> bool {
        self == &Self::Phi
    }

    pub fn is_call(&self) -> bool {
        self == &Self::Call
    }

    pub fn is_invoke(&self) -> bool {
        self == &Self::Invoke
    }

    pub fn has_side_effects(&self) -> bool {
        self.is_load()
            || self.is_store()
            || self.is_alloca()
            || self.is_phi()
            || self.is_call()
            || self.is_invoke()
            || self.is_terminator()
    }
}

macro_rules! as_inst {
    ($name:ident, $inst:ident) => {
        pub fn $name(&self) -> Option<&$inst> {
            match self {
                Self::$inst(x) => Some(x),
                _ => None,
            }
        }
    };
    (mut $name:ident, $inst:ident) => {
        pub fn $name(&mut self) -> Option<&mut $inst> {
            match self {
                Self::$inst(x) => Some(x),
                _ => None,
            }
        }
    };
}

impl Operand {
    pub fn args(&self) -> &[ValueId] {
        match self {
            Self::Alloca(_) => &[],
            Self::Phi(Phi { args, .. }) => args.as_slice(),
            Self::Ret(Ret { val, .. }) if val.is_none() => &[],
            Self::Ret(Ret { val, .. }) => slice::from_ref(val.as_ref().unwrap()),
            Self::Load(Load { addr, .. }) => slice::from_ref(addr),
            Self::Store(Store { args, .. }) => args,
            Self::InsertValue(InsertValue { args, .. }) => args,
            Self::ExtractValue(ExtractValue { args, .. }) => args,
            Self::IntBinary(IntBinary { args, .. }) => args,
            Self::ICmp(ICmp { args, .. }) => args,
            Self::Cast(Cast { arg, .. }) => slice::from_ref(arg),
            Self::GetElementPtr(GetElementPtr { args, .. }) => args.as_slice(),
            Self::Call(Call { args, .. }) | Self::Invoke(Invoke { args, .. }) => args.as_slice(),
            Self::LandingPad(LandingPad { .. }) => &[],
            Self::Resume(Resume { arg, .. }) => slice::from_ref(arg),
            Self::Br(Br { .. }) => &[],
            Self::CondBr(CondBr { arg, .. }) => slice::from_ref(arg),
            Self::Invalid => &[],
        }
    }

    pub fn args_mut(&mut self) -> &mut [ValueId] {
        match self {
            Self::Alloca(_) => &mut [],
            Self::Phi(Phi { args, .. }) => args.as_mut_slice(),
            Self::Ret(Ret { val, .. }) if val.is_none() => &mut [],
            Self::Ret(Ret { val, .. }) => slice::from_mut(val.as_mut().unwrap()),
            Self::Load(Load { addr, .. }) => slice::from_mut(addr),
            Self::Store(Store { args, .. }) => args,
            Self::InsertValue(InsertValue { args, .. }) => args,
            Self::ExtractValue(ExtractValue { args, .. }) => args,
            Self::IntBinary(IntBinary { args, .. }) => args,
            Self::ICmp(ICmp { args, .. }) => args,
            Self::Cast(Cast { arg, .. }) => slice::from_mut(arg),
            Self::GetElementPtr(GetElementPtr { args, .. }) => args.as_mut_slice(),
            Self::Call(Call { args, .. }) | Self::Invoke(Invoke { args, .. }) => args.as_mut(),
            Self::LandingPad(LandingPad { .. }) => &mut [],
            Self::Resume(Resume { arg, .. }) => slice::from_mut(arg),
            Self::Br(Br { .. }) => &mut [],
            Self::CondBr(CondBr { arg, .. }) => slice::from_mut(arg),
            Self::Invalid => &mut [],
        }
    }

    pub fn types(&self) -> &[TypeId] {
        match self {
            Self::Alloca(Alloca { tys, .. }) => tys,
            Self::Phi(Phi { ty, .. }) => slice::from_ref(ty),
            Self::Ret(Ret { ty, .. }) => slice::from_ref(ty),
            Self::Load(Load { tys, .. }) => tys,
            Self::Store(Store { .. }) => &[],
            Self::InsertValue(InsertValue { tys, .. }) => tys,
            Self::ExtractValue(ExtractValue { ty, .. }) => slice::from_ref(ty),
            Self::IntBinary(IntBinary { ty, .. }) => slice::from_ref(ty),
            Self::ICmp(ICmp { ty, .. }) => slice::from_ref(ty),
            Self::Cast(Cast { tys, .. }) => tys,
            Self::GetElementPtr(GetElementPtr { tys, .. }) => tys.as_slice(),
            Self::Call(Call { tys, .. }) | Self::Invoke(Invoke { tys, .. }) => tys.as_slice(),
            Self::LandingPad(LandingPad { ty }) => slice::from_ref(ty),
            Self::Resume(Resume { ty, .. }) => slice::from_ref(ty),
            Self::Br(Br { .. }) => &[],
            Self::CondBr(CondBr { .. }) => &[],
            Self::Invalid => &[],
        }
    }

    pub fn blocks(&self) -> &[BasicBlockId] {
        match self {
            Self::Phi(Phi { blocks, .. }) => blocks,
            Self::Br(Br { block }) => slice::from_ref(block),
            Self::CondBr(CondBr { blocks, .. }) => blocks,
            Self::Invoke(Invoke { blocks, .. }) => blocks,
            _ => &[],
        }
    }

    pub fn call_result_ty(&self) -> Option<TypeId> {
        match self {
            Self::Call(Call { tys, .. }) | Self::Invoke(Invoke { tys, .. }) => Some(tys[0]),
            _ => None,
        }
    }

    as_inst!(as_alloca, Alloca);
    as_inst!(as_store, Store);
    as_inst!(as_load, Load);
    as_inst!(as_phi, Phi);
    as_inst!(mut as_phi_mut, Phi);
    as_inst!(as_condbr, CondBr);
}

impl Alloca {
    pub fn ty(&self) -> TypeId {
        self.tys[0]
    }
}

impl Store {
    pub fn dst_val(&self) -> ValueId {
        self.args[1]
    }

    pub fn src_val(&self) -> ValueId {
        self.args[0]
    }
}

impl Load {
    pub fn src_val(&self) -> ValueId {
        self.addr
    }
}

impl Phi {
    pub fn blocks_mut(&mut self) -> &mut Vec<BasicBlockId> {
        &mut self.blocks
    }

    pub fn args_mut(&mut self) -> &mut Vec<ValueId> {
        &mut self.args
    }
}

impl fmt::Debug for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Opcode::Alloca => "alloca",
                Opcode::Phi => "phi",
                Opcode::Load => "load",
                Opcode::Store => "store",
                Opcode::InsertValue => "insertvalue",
                Opcode::ExtractValue => "extractvalue",
                Opcode::Add => "add",
                Opcode::Sub => "sub",
                Opcode::Mul => "mul",
                Opcode::SDiv => "sdiv",
                Opcode::SRem => "srem",
                Opcode::And => "and",
                Opcode::LShr => "lshr",
                Opcode::ICmp => "icmp",
                Opcode::Sext => "sext",
                Opcode::Zext => "zext",
                Opcode::Bitcast => "bitcast",
                Opcode::Trunc => "trunc",
                Opcode::IntToPtr => "inttoptr",
                Opcode::GetElementPtr => "getelementptr",
                Opcode::Call => "call",
                Opcode::Invoke => "invoke",
                Opcode::LandingPad => "landingpad",
                Opcode::Resume => "resume",
                Opcode::Br | Opcode::CondBr => "br",
                Opcode::Ret => "ret",
                Opcode::Invalid => "INVALID",
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
