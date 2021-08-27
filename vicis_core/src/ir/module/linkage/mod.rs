pub mod parser;

pub use parser::parse;

use std::fmt;

#[derive(Clone, Copy)]
pub enum Linkage {
    Private,
    Internal,
    External,
    ExternalWeak,
    AvailableExternally,
    LinkOnceAny,
    LinkOnceODR,
    LinkOnceODRAutoHide,
    WeakAny,
    WeakODR,
    Common,
    Appending,
    DLLImport,
    DLLExport,
    Ghost,
    LinkerPrivate,
    LinkerPrivateWeak,
}

impl fmt::Debug for Linkage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Private => write!(f, "private"),
            Self::Internal => write!(f, "internal"),
            Self::External => write!(f, "external"),
            Self::ExternalWeak => write!(f, "externalweak"),
            Self::AvailableExternally => write!(f, "availableexternally"),
            Self::LinkOnceAny => write!(f, "linkonceany"),
            Self::LinkOnceODR => write!(f, "linkonceodr"),
            Self::LinkOnceODRAutoHide => write!(f, "linkonceodrautohide"),
            Self::WeakAny => write!(f, "weakany"),
            Self::WeakODR => write!(f, "weakodr"),
            Self::Common => write!(f, "common"),
            Self::Appending => write!(f, "appending"),
            Self::DLLImport => write!(f, "dllimport"),
            Self::DLLExport => write!(f, "dllexport"),
            Self::Ghost => write!(f, "ghost"),
            Self::LinkerPrivate => write!(f, "linkerprivate"),
            Self::LinkerPrivateWeak => write!(f, "linkerprivateweak"),
        }
    }
}
