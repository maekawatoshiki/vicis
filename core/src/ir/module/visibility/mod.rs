use std::fmt;

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub enum Visibility {
    Default,
    Hidden,
    Protected,
}

impl Default for Visibility {
    fn default() -> Self {
        Visibility::Default
    }
}

impl fmt::Debug for Visibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Default => write!(f, "default"),
            Self::Hidden => write!(f, "hidden"),
            Self::Protected => write!(f, "protected"),
        }
    }
}
