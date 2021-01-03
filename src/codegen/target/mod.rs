pub mod x86_64;

pub trait Target {
    type InstData;

    fn select_patterns() -> Vec<()>;
}
