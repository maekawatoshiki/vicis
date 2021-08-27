pub mod parser;

pub use parser::parse;

use super::{
    function::{data::Data, instruction::InstructionId},
    module::name::Name,
    types::{TypeId, Types},
    util::escape,
};
use id_arena::Id;
use std::{fmt, str};

pub type ValueId = Id<Value>;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Instruction(InstructionId),
    Argument(usize),
    Constant(ConstantData),
    InlineAsm(InlineAsm),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConstantData {
    Undef,
    AggregateZero,
    Null,
    Int(ConstantInt),
    Array(ConstantArray),
    Struct(ConstantStruct),
    Expr(ConstantExpr), // TODO: Boxing?
    GlobalRef(Name),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConstantInt {
    Int1(bool),
    Int8(i8),
    Int32(i32),
    Int64(i64),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConstantArray {
    pub elem_ty: TypeId,
    pub elems: Vec<ConstantData>,
    pub is_string: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConstantStruct {
    pub elems_ty: Vec<TypeId>,
    pub elems: Vec<ConstantData>,
    pub is_packed: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConstantExpr {
    GetElementPtr {
        inbounds: bool,
        tys: Vec<TypeId>,
        args: Vec<ConstantData>,
    },
    Bitcast {
        tys: [TypeId; 2],
        arg: Box<ConstantData>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct InlineAsm {
    pub body: String,
    pub constraints: String,
    pub sideeffect: bool,
}

impl Value {
    pub fn undef() -> Self {
        Self::Constant(ConstantData::Undef)
    }

    pub fn as_inst(&self) -> &InstructionId {
        match self {
            Self::Instruction(id) => id,
            _ => panic!(),
        }
    }

    pub fn to_string(&self, _data: &Data, types: &Types) -> String {
        match self {
            Self::Constant(c) => c.to_string(types),
            Self::Instruction(id) => {
                format!("%I{}", id.index())
            }
            Self::Argument(n) => format!("%A{}", n),
            Self::InlineAsm(InlineAsm {
                body,
                constraints,
                sideeffect,
            }) => {
                format!(
                    "asm {}\"{}\", \"{}\"",
                    if *sideeffect { "sideeffect " } else { "" },
                    constraints,
                    body
                )
            }
        }
    }
}

impl ConstantData {
    pub fn to_string(&self, types: &Types) -> String {
        match self {
            Self::Undef => "undef".to_string(),
            Self::AggregateZero => "AggregateZero".to_string(),
            Self::Null => "null".to_string(),
            Self::Int(i) => i.to_string(),
            Self::Array(a) => a.to_string(types),
            Self::Struct(s) => s.to_string(types),
            Self::Expr(e) => e.to_string(types),
            Self::GlobalRef(name) => format!("@{:?}", name),
        }
    }

    pub fn as_int(&self) -> &ConstantInt {
        match self {
            Self::Int(i) => i,
            _ => panic!(),
        }
    }

    pub fn as_global_ref(&self) -> &Name {
        match self {
            Self::GlobalRef(name) => name,
            _ => panic!(),
        }
    }

    pub fn as_array(&self) -> &ConstantArray {
        match self {
            Self::Array(a) => a,
            _ => panic!(),
        }
    }
}

impl ConstantInt {
    pub fn as_i8(&self) -> &i8 {
        match self {
            Self::Int8(i) => i,
            _ => panic!(),
        }
    }

    // TODO: Generics?
    pub fn cast_to_usize(self) -> usize {
        match self {
            Self::Int1(i) => i as usize,
            Self::Int8(i) => i as usize,
            Self::Int32(i) => i as usize,
            Self::Int64(i) => i as usize,
        }
    }
}

impl ConstantArray {
    pub fn to_string(&self, types: &Types) -> String {
        if self.is_string {
            return format!(
                "c\"{}\"",
                escape(
                    str::from_utf8(
                        self.elems
                            .iter()
                            .map(|i| *i.as_int().as_i8() as u8)
                            .collect::<Vec<u8>>()
                            .as_slice(),
                    )
                    .unwrap()
                )
            );
        }

        format!(
            "{}]",
            self.elems
                .iter()
                .fold("[".to_string(), |acc, e| {
                    format!(
                        "{}{} {}, ",
                        acc,
                        types.to_string(self.elem_ty),
                        e.to_string(types)
                    )
                })
                .trim_end_matches(", ")
        )
    }
}

impl ConstantStruct {
    pub fn to_string(&self, types: &Types) -> String {
        format!(
            "{}{{ {} }}{}",
            if self.is_packed { "<" } else { "" },
            self.elems
                .iter()
                .zip(self.elems_ty.iter())
                .fold("".to_string(), |acc, (e, &et)| {
                    format!("{}{} {}, ", acc, types.to_string(et), e.to_string(types))
                })
                .trim_end_matches(", "),
            if self.is_packed { ">" } else { "" }
        )
    }
}

impl ConstantExpr {
    pub fn to_string(&self, types: &Types) -> String {
        match self {
            Self::GetElementPtr {
                inbounds,
                tys,
                args,
            } => {
                format!(
                    "getelementptr {}({}, {})",
                    if *inbounds { "inbounds " } else { "" },
                    types.to_string(tys[0]),
                    tys[1..]
                        .iter()
                        .zip(args.iter())
                        .fold("".to_string(), |acc, (ty, arg)| {
                            format!("{}{} {}, ", acc, types.to_string(*ty), arg.to_string(types))
                        })
                        .trim_end_matches(", ")
                )
            }
            Self::Bitcast { tys, arg } => {
                format!(
                    "bitcast ({} {} to {})",
                    types.to_string(tys[0]),
                    arg.to_string(types),
                    types.to_string(tys[1]),
                )
            }
        }
    }
}

impl From<i32> for Value {
    fn from(i: i32) -> Self {
        Self::Constant(ConstantInt::Int32(i).into())
    }
}

impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Self::Constant(ConstantInt::Int64(i).into())
    }
}

impl From<ConstantInt> for ConstantData {
    fn from(i: ConstantInt) -> Self {
        Self::Int(i)
    }
}

impl From<ConstantInt> for Value {
    fn from(i: ConstantInt) -> Self {
        Self::Constant(i.into())
    }
}

impl fmt::Display for ConstantInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int1(i) => write!(f, "{}", i),
            Self::Int8(i) => write!(f, "{}", i),
            Self::Int32(i) => write!(f, "{}", i),
            Self::Int64(i) => write!(f, "{}", i),
        }
    }
}
