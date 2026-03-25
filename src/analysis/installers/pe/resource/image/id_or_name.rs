#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdOrName {
    Id(u32),
    Name(String),
}

impl Default for IdOrName {
    fn default() -> Self {
        Self::Id(0)
    }
}
