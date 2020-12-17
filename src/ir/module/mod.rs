pub mod parser;

#[derive(Debug, Clone)]
struct Target {
    triple: String,
    dayalayout: String,
}

#[derive(Debug, Clone)]
pub struct Module {
    name: String,
    source_filename: String,
    target: Target,
}
