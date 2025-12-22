use std::{
    fmt, io,
    io::{Read, Seek},
};

use super::{
    super::{ResourceDirectory, ResourceDirectoryEntryData},
    ImageResourceDirectoryEntry,
};

/// An [`ImageResourceDirectoryEntry`] with a resolved name.
#[derive(Clone, Eq, PartialEq)]
pub struct NamedImageResourceDirectoryEntry {
    name: String,
    entry: ImageResourceDirectoryEntry,
}

impl NamedImageResourceDirectoryEntry {
    /// Returns a new [`NamedImageResourceDirectoryEntry`] from a [`String`] and an
    /// [`ImageResourceDirectoryEntry`].
    #[inline]
    pub const fn new(name: String, entry: ImageResourceDirectoryEntry) -> Self {
        Self { name, entry }
    }

    /// Returns the name of the [`NamedImageResourceDirectoryEntry`].
    pub const fn name(&self) -> &str {
        self.name.as_str()
    }

    #[inline]
    pub const fn entry(&self) -> ImageResourceDirectoryEntry {
        self.entry
    }

    /// Returns the raw ID of the directory entry.
    #[inline]
    pub const fn raw_id(&self) -> u32 {
        self.entry().name_or_id()
    }

    /// Returns true if this directory entry is a table.
    #[inline]
    pub const fn is_table(&self) -> bool {
        self.entry().is_table()
    }

    #[inline]
    pub const fn data_offset(&self) -> u32 {
        self.entry().data_offset()
    }

    /// Returns the data associated to this directory entry.
    pub fn data<R>(
        &self,
        section: &mut ResourceDirectory<R>,
    ) -> io::Result<ResourceDirectoryEntryData>
    where
        R: Read + Seek,
    {
        self.entry().data(section)
    }
}

impl fmt::Debug for NamedImageResourceDirectoryEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("NamedImageResourceDirectoryEntry")
            .field("Name", &self.name())
            .field("Table", &self.is_table())
            .field("Offset", &self.data_offset())
            .finish()
    }
}
