#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum CallConvKind {
    SystemV,
    AAPCS64,
}
