use crate::ir::{
    module::name::Name,
    types::{self, Type, Typed, Types},
    util::escape,
};

#[derive(Debug, Clone, PartialEq)]
pub enum ConstantValue {
    Undef(Type),
    AggregateZero(Type),
    Null(Type),
    Int(ConstantInt),
    Array(ConstantArray),
    Struct(ConstantStruct),
    Expr(ConstantExpr), // TODO: Boxing?
    GlobalRef(Name, Type),
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
    pub ty: Type,
    pub elem_ty: Type,
    pub elems: Vec<ConstantValue>,
    pub is_string: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConstantStruct {
    pub ty: Type,
    pub elems_ty: Vec<Type>,
    pub elems: Vec<ConstantValue>,
    pub is_packed: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConstantExpr {
    GetElementPtr {
        inbounds: bool,
        tys: Vec<Type>,
        args: Vec<ConstantValue>,
    },
    Bitcast {
        tys: [Type; 2],
        arg: Box<ConstantValue>,
    },
}

impl ConstantValue {
    pub fn to_string(&self, types: &Types) -> String {
        match self {
            Self::Undef(_) => "undef".to_string(),
            Self::AggregateZero(_) => "zeroinitializer".to_string(),
            Self::Null(_) => "null".to_string(),
            Self::Int(i) => i.to_string(),
            Self::Array(a) => a.to_string(types),
            Self::Struct(s) => s.to_string(types),
            Self::Expr(e) => e.to_string(types),
            Self::GlobalRef(name, _) => format!("@{:?}", name),
        }
    }

    pub fn as_int(&self) -> Option<&ConstantInt> {
        match self {
            Self::Int(i) => Some(i),
            _ => None,
        }
    }

    pub fn as_global_ref(&self) -> &Name {
        match self {
            Self::GlobalRef(name, _) => name,
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

    pub fn as_i32(&self) -> Option<&i32> {
        match self {
            Self::Int32(i) => Some(i),
            _ => None,
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

    pub fn cast_to_i64(self) -> i64 {
        match self {
            Self::Int1(i) => i as i64,
            Self::Int8(i) => i as i64,
            Self::Int32(i) => i as i64,
            Self::Int64(i) => i as i64,
        }
    }

    pub fn is_zero(&self) -> bool {
        match self {
            Self::Int1(i) => !(*i),
            Self::Int8(i) => *i == 0,
            Self::Int32(i) => *i == 0,
            Self::Int64(i) => *i == 0,
        }
    }
}

impl ConstantArray {
    pub fn to_string(&self, types: &Types) -> String {
        if self.is_string {
            return format!(
                "c\"{}\"",
                escape(
                    std::str::from_utf8(
                        self.elems
                            .iter()
                            .map(|i| *i.as_int().unwrap().as_i8() as u8)
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

impl Typed for ConstantValue {
    fn ty(&self) -> Type {
        match self {
            Self::Undef(ty) => *ty,
            Self::AggregateZero(ty) => *ty,
            Self::Null(ty) => *ty,
            Self::Int(i) => i.ty(),
            Self::Array(a) => a.ty(),
            Self::Struct(s) => s.ty(),
            Self::Expr(e) => e.ty(),
            Self::GlobalRef(_name, ty) => *ty,
        }
    }
}

impl Typed for ConstantInt {
    fn ty(&self) -> Type {
        match self {
            Self::Int1(_) => types::I1,
            Self::Int8(_) => types::I8,
            Self::Int32(_) => types::I32,
            Self::Int64(_) => types::I64,
        }
    }
}

impl Typed for ConstantArray {
    fn ty(&self) -> Type {
        self.ty
    }
}

impl Typed for ConstantStruct {
    fn ty(&self) -> Type {
        self.ty
    }
}

impl Typed for ConstantExpr {
    fn ty(&self) -> Type {
        match self {
            Self::GetElementPtr { tys, .. } => tys[0],
            Self::Bitcast { tys, .. } => tys[1],
        }
    }
}

impl From<ConstantInt> for ConstantValue {
    fn from(i: ConstantInt) -> Self {
        Self::Int(i)
    }
}

impl From<ConstantInt> for super::Value {
    fn from(i: ConstantInt) -> Self {
        Self::Constant(i.into())
    }
}

impl std::fmt::Display for ConstantInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int1(i) => write!(f, "{}", i),
            Self::Int8(i) => write!(f, "{}", i),
            Self::Int32(i) => write!(f, "{}", i),
            Self::Int64(i) => write!(f, "{}", i),
        }
    }
}
