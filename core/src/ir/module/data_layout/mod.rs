#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct DataLayout(pub String);

impl DataLayout {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for DataLayout {
    fn from(s: String) -> Self {
        DataLayout(s)
    }
}
