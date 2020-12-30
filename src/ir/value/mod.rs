pub mod parser;

pub use parser::parse;

use super::{
    function::Data,
    instruction::InstructionId,
    module::name::Name,
    types::{TypeId, Types},
};
use id_arena::Id;

pub type ValueId = Id<Value>;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Instruction(InstructionId),
    Argument(usize),
    Constant(ConstantData),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConstantData {
    Int(ConstantInt),
    Array(ConstantArray),
    Expr(ConstantExpr), // TODO: Boxing?
    GlobalRef(Name),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConstantInt {
    Int8(i8),
    Int32(i32),
    Int64(i64),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConstantArray {
    pub elem_ty: TypeId,
    pub elems: Vec<ConstantData>,
    pub is_string: bool, // Int32(i32),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConstantExpr {
    GetElementPtr {
        inbounds: bool,
        tys: Vec<TypeId>,
        args: Vec<ConstantData>,
    },
}

impl Value {
    pub fn to_string(&self, _data: &Data, types: &Types) -> String {
        match self {
            Self::Constant(c) => c.to_string(types),
            Self::Instruction(id) => {
                format!("%I{}", id.index())
            }
            Self::Argument(n) => format!("%A{}", n),
            // Self::UnresolvedGlobalName(n) => format!("@{:?}", n),
        }
    }
}

impl ConstantData {
    pub fn to_string(&self, types: &Types) -> String {
        match self {
            Self::Int(i) => i.to_string(),
            Self::Array(a) => a.to_string(types),
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
}

impl ConstantInt {
    pub fn to_string(&self) -> String {
        match self {
            Self::Int8(i) => format!("{}", i),
            Self::Int32(i) => format!("{}", i),
            Self::Int64(i) => format!("{}", i),
        }
    }

    pub fn as_i8(&self) -> &i8 {
        match self {
            Self::Int8(i) => i,
            _ => panic!(),
        }
    }

    // TODO: Generics?
    pub fn cast_to_usize(self) -> usize {
        match self {
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
                ::std::str::from_utf8(
                    self.elems
                        .iter()
                        .map(|i| *i.as_int().as_i8() as u8)
                        .collect::<Vec<u8>>()
                        .as_slice()
                )
                .unwrap()
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
        }
    }
}
