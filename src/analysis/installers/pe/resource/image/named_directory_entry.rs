use std::{
    io,
    io::{Read, Seek},
};

use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, LittleEndian, U32};

use crate::analysis::installers::pe::resource::{
    ImageResourceDataEntry, ImageResourceDirectoryEntry, ResourceDirectory, ResourceDirectoryTable,
    directory_entry_data::ResourceDirectoryEntryData,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamedImageResourceDirectoryEntry {
    name: String,
    entry: ImageResourceDirectoryEntry,
}

impl NamedImageResourceDirectoryEntry {
    pub fn new(
        name: String,
        entry: ImageResourceDirectoryEntry,
    ) -> NamedImageResourceDirectoryEntry {
        Self { name, entry }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn entry(&self) -> ImageResourceDirectoryEntry {
        self.entry
    }

    /// Returns the data associated to this directory entry.
    pub fn data<R>(
        self,
        section: &mut ResourceDirectory<R>,
    ) -> io::Result<ResourceDirectoryEntryData>
    where
        R: Read + Seek,
    {
        self.entry.data(section)
    }
}
