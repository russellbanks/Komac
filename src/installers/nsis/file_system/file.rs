use std::{borrow::Borrow, hash::Hash};

use chrono::{DateTime, Utc};
use compact_str::CompactString;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct File {
    pub name: CompactString,
    pub modified: DateTime<Utc>,
}

impl File {
    pub fn new<T, D>(name: T, modified: D) -> Self
    where
        T: Into<CompactString>,
        D: Into<DateTime<Utc>>,
    {
        Self {
            name: name.into(),
            modified: modified.into(),
        }
    }

    #[inline]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    #[inline]
    pub const fn modified(&self) -> &DateTime<Utc> {
        &self.modified
    }
}

impl Borrow<str> for File {
    #[inline]
    fn borrow(&self) -> &str {
        self.name.borrow()
    }
}
