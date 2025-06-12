use std::fmt;

use chrono::{DateTime, Utc};
use compact_str::CompactString;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Item {
    File {
        name: CompactString,
        modified_at: Option<DateTime<Utc>>,
        position: usize,
    },
    Directory(CompactString),
}

impl Item {
    #[inline]
    pub const fn new_root() -> Self {
        Self::Directory(CompactString::const_new("/"))
    }

    pub fn new_directory<T>(name: T) -> Self
    where
        T: Into<CompactString>,
    {
        Self::Directory(name.into())
    }

    pub fn new_file<T, D, P>(name: T, modified_at: D, position: P) -> Self
    where
        T: Into<CompactString>,
        D: Into<Option<DateTime<Utc>>>,
        P: Into<usize>,
    {
        Self::File {
            name: name.into(),
            modified_at: modified_at.into(),
            position: position.into(),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::File { name, .. } | Self::Directory(name) => name.as_str(),
        }
    }

    pub const fn modified_at(&self) -> Option<&DateTime<Utc>> {
        match self {
            Self::File { modified_at, .. } => modified_at.as_ref(),
            Self::Directory(_) => None,
        }
    }

    pub const fn position(&self) -> Option<usize> {
        match self {
            Self::File { position, .. } => Some(*position + size_of::<u32>()),
            Self::Directory(_) => None,
        }
    }

    #[inline]
    pub const fn is_file(&self) -> bool {
        matches!(self, Self::File { .. })
    }

    #[inline]
    pub const fn is_directory(&self) -> bool {
        matches!(self, Self::Directory { .. })
    }
}

impl fmt::Debug for Item {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::File {
                name,
                modified_at,
                position,
            } => {
                if let Some(modified_at) = modified_at {
                    f.debug_struct("File")
                        .field("name", name)
                        .field("modified_at", modified_at)
                        .field("position", position)
                        .finish()
                } else {
                    f.debug_tuple("File").field(name).finish()
                }
            }
            Self::Directory(name) => f.debug_tuple("Directory").field(name).finish(),
        }
    }
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.name().fmt(f)
    }
}
