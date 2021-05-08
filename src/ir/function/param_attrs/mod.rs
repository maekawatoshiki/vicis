pub mod parser;

use crate::ir::types::{TypeId, Types};
use std::fmt;

#[derive(PartialEq, Eq, Clone)]
pub enum ParameterAttribute {
    ZeroExt,
    SignExt,
    InReg,
    ByVal,
    InAlloca,
    SRet(Option<TypeId>),
    Alignment(u64),
    ReadOnly,
    NoAlias,
    NoCapture,
    NoFree,
    Nest,
    Returned,
    NonNull,
    Dereferenceable(u64),
    DereferenceableOrNull(u64),
    SwiftSelf,
    SwiftError,
    ImmArg,
    StringAttribute { kind: String, value: String },
    Ref(u32),
    UnknownAttribute,
}

impl ParameterAttribute {
    pub fn to_string(&self, types: &Types) -> String {
        match self {
            Self::ZeroExt => format!("zeroext"),
            Self::SignExt => format!("signext"),
            Self::InReg => format!("inreg"),
            Self::ByVal => format!("byval"),
            Self::InAlloca => format!("inalloca"),
            Self::SRet(None) => format!("sret"),
            Self::SRet(Some(ty)) => format!("sret({})", types.to_string(*ty)),
            Self::Alignment(i) => format!("align {}", i),
            Self::ReadOnly => format!("readonly"),
            Self::NoAlias => format!("noalias"),
            Self::NoCapture => format!("nocapture"),
            Self::NoFree => format!("nofree"),
            Self::Nest => format!("nest"),
            Self::Returned => format!("returned"),
            Self::NonNull => format!("nonnull"),
            Self::Dereferenceable(i) => format!("dereferenceable({})", i),
            Self::DereferenceableOrNull(i) => format!("dereferenceableornull({})", i),
            Self::SwiftSelf => format!("swiftself"),
            Self::SwiftError => format!("swifterror"),
            Self::ImmArg => format!("immarg"),
            Self::StringAttribute { kind, value } => format!("\"{}\"=\"{}\"", kind, value),
            Self::Ref(i) => format!("#{}", i),
            Self::UnknownAttribute => format!(""),
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
            Self::Dereferenceable(i) => write!(f, "dereferenceable({})", i),
            Self::DereferenceableOrNull(i) => write!(f, "dereferenceableornull({})", i),
            Self::SwiftSelf => write!(f, "swiftself"),
            Self::SwiftError => write!(f, "swifterror"),
            Self::ImmArg => write!(f, "immarg"),
            Self::StringAttribute { kind, value } => write!(f, "\"{}\"=\"{}\"", kind, value),
            Self::Ref(i) => write!(f, "#{}", i),
            Self::UnknownAttribute => write!(f, ""),
        }
    }
}
