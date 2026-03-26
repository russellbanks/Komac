use std::fmt;

use chrono::{DateTime, Utc};
use compact_str::CompactString;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FsEntry {
    File {
        name: CompactString,
        modified_at: Option<DateTime<Utc>>,
        position: u64,
    },
    Directory(CompactString),
}

impl FsEntry {
    /// Creates a new root [`FsEntry`].
    ///
    /// This will be a [directory] with a path of `/`.
    ///
    /// [Directory]: Self::Directory
    #[inline]
    pub const fn new_root() -> Self {
        Self::Directory(CompactString::const_new("/"))
    }

    /// Creates a new directory [`FsEntry`] from a name.
    pub fn new_directory<T>(name: T) -> Self
    where
        T: Into<CompactString>,
    {
        Self::Directory(name.into())
    }

    /// Creates a new file [`FsEntry`] from a name, an optional modified at [datetime], and a
    /// position.
    ///
    /// [datetime]: DateTime<Utc>
    pub fn new_file<T, D, P>(name: T, modified_at: D, position: P) -> Self
    where
        T: Into<CompactString>,
        D: Into<Option<DateTime<Utc>>>,
        P: Into<u64>,
    {
        Self::File {
            name: name.into(),
            modified_at: modified_at.into(),
            position: position.into(),
        }
    }

    /// Returns the entry's name as a string slice.
    pub fn name(&self) -> &str {
        match self {
            Self::File { name, .. } | Self::Directory(name) => name.as_str(),
        }
    }

    /// Returns the modified at [datetime] if this entry is a file, or None if it is a directory.
    ///
    /// [datetime]: DateTime<Utc>
    pub const fn modified_at(&self) -> Option<&DateTime<Utc>> {
        match self {
            Self::File { modified_at, .. } => modified_at.as_ref(),
            Self::Directory(_) => None,
        }
    }

    /// Returns the position if this entry is a file, or None if it is a directory.
    pub const fn position(&self) -> Option<u64> {
        match self {
            Self::File { position, .. } => Some(*position + size_of::<u32>() as u64),
            Self::Directory(_) => None,
        }
    }

    /// Returns `true` if this entry is a file.
    #[must_use]
    #[inline]
    pub const fn is_file(&self) -> bool {
        matches!(self, Self::File { .. })
    }

    /// Returns `true` if this entry is a directory.
    #[must_use]
    #[inline]
    pub const fn is_directory(&self) -> bool {
        matches!(self, Self::Directory { .. })
    }
}

impl fmt::Debug for FsEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::File {
                name,
                modified_at,
                position,
            } => {
                if let Some(modified_at) = modified_at {
                    f.debug_struct("File")
                        .field("Name", name)
                        .field("ModifiedAt", modified_at)
                        .field("Position", position)
                        .finish()
                } else {
                    f.debug_tuple("File").field(name).finish()
                }
            }
            Self::Directory(name) => f.debug_tuple("Directory").field(name).finish(),
        }
    }
}

impl fmt::Display for FsEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.name().fmt(f)
    }
}
