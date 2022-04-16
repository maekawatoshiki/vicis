use std::fmt;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum PreemptionSpecifier {
    DsoPreemptable,
    DsoLocal,
}

impl Default for PreemptionSpecifier {
    fn default() -> Self {
        PreemptionSpecifier::DsoPreemptable
    }
}

impl fmt::Debug for PreemptionSpecifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DsoPreemptable => write!(f, "dso_preemptable"),
            Self::DsoLocal => write!(f, "dso_local"),
        }
    }
}
