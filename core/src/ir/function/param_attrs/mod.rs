use crate::ir::types::{Type, Types};
use std::fmt;

#[derive(PartialEq, Eq, Clone)]
pub enum ParameterAttribute {
    ZeroExt,
    SignExt,
    InReg,
    ByVal,
    InAlloca,
    SRet(Option<Type>),
    Alignment(u64),
    ReadOnly,
    NoAlias,
    NoCapture,
    NoFree,
    Nest,
    Returned,
    NonNull,
    NoUndef,
    Dereferenceable(u64),
    DereferenceableOrNull(u64),
    SwiftSelf,
    SwiftError,
    ImmArg,
    WriteOnly,
    StringAttribute { kind: String, value: String },
    Ref(u32),
    UnknownAttribute,
}

impl ParameterAttribute {
    pub fn to_string(&self, types: &Types) -> String {
        match self {
            Self::ZeroExt => "zeroext".to_string(),
            Self::SignExt => "signext".to_string(),
            Self::InReg => "inreg".to_string(),
            Self::ByVal => "byval".to_string(),
            Self::InAlloca => "inalloca".to_string(),
            Self::SRet(None) => "sret".to_string(),
            Self::SRet(Some(ty)) => format!("sret({})", types.to_string(*ty)),
            Self::Alignment(i) => format!("align {}", i),
            Self::ReadOnly => "readonly".to_string(),
            Self::NoAlias => "noalias".to_string(),
            Self::NoCapture => "nocapture".to_string(),
            Self::NoFree => "nofree".to_string(),
            Self::Nest => "nest".to_string(),
            Self::Returned => "returned".to_string(),
            Self::NonNull => "nonnull".to_string(),
            Self::NoUndef => "noundef".to_string(),
            Self::Dereferenceable(i) => format!("dereferenceable({})", i),
            Self::DereferenceableOrNull(i) => format!("dereferenceableornull({})", i),
            Self::SwiftSelf => "swiftself".to_string(),
            Self::SwiftError => "swifterror".to_string(),
            Self::ImmArg => "immarg".to_string(),
            Self::WriteOnly => "writeonly".to_string(),
            Self::StringAttribute { kind, value } => format!("\"{}\"=\"{}\"", kind, value),
            Self::Ref(i) => format!("#{}", i),
            Self::UnknownAttribute => "".to_string(),
        }
    }
}

impl fmt::Debug for ParameterAttribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroExt => write!(f, "zeroext"),
            Self::SignExt => write!(f, "signext"),
            Self::InReg => write!(f, "inreg"),
            Self::ByVal => write!(f, "byval"),
            Self::InAlloca => write!(f, "inalloca"),
            Self::SRet(None) => write!(f, "sret"),
            Self::SRet(Some(_)) => write!(f, "sret(type)"),
            Self::Alignment(i) => write!(f, "align {}", i),
            Self::ReadOnly => write!(f, "readonly"),
            Self::NoAlias => write!(f, "noalias"),
            Self::NoCapture => write!(f, "nocapture"),
            Self::NoFree => write!(f, "nofree"),
            Self::Nest => write!(f, "nest"),
            Self::Returned => write!(f, "returned"),
            Self::NonNull => write!(f, "nonnull"),
            Self::NoUndef => write!(f, "noundef"),
            Self::Dereferenceable(i) => write!(f, "dereferenceable({})", i),
            Self::DereferenceableOrNull(i) => write!(f, "dereferenceableornull({})", i),
            Self::SwiftSelf => write!(f, "swiftself"),
            Self::SwiftError => write!(f, "swifterror"),
            Self::ImmArg => write!(f, "immarg"),
            Self::WriteOnly => write!(f, "writeonly"),
            Self::StringAttribute { kind, value } => write!(f, "\"{}\"=\"{}\"", kind, value),
            Self::Ref(i) => write!(f, "#{}", i),
            Self::UnknownAttribute => write!(f, ""),
        }
    }
}
