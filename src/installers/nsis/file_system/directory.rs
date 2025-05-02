use compact_str::CompactString;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Directory {
    name: CompactString,
}

impl Directory {
    pub fn new_root() -> Self {
        Self {
            name: CompactString::new("/"),
        }
    }

    pub fn new<T>(name: T) -> Self
    where
        T: Into<CompactString>,
    {
        Self { name: name.into() }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}
