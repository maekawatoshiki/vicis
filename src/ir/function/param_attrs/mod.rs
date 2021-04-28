pub mod parser;

use std::fmt;

#[derive(PartialEq, Eq, Clone)]
pub enum ParameterAttribute {
    ZeroExt,
    SignExt,
    InReg,
    ByVal,
    InAlloca,
    SRet,
    Alignment(u64),
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

impl fmt::Debug for ParameterAttribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroExt => write!(f, "zeroext"),
            Self::SignExt => write!(f, "signext"),
            Self::InReg => write!(f, "inreg"),
            Self::ByVal => write!(f, "byval"),
            Self::InAlloca => write!(f, "inalloca"),
            Self::SRet => write!(f, "sret"),
            Self::Alignment(i) => write!(f, "align({})", i),
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
