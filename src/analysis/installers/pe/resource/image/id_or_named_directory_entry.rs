use std::{
    io,
    io::{Read, Seek},
};

use super::{
    super::{ResourceDirectory, directory_entry_data::ResourceDirectoryEntryData},
    IdImageResourceDirectoryEntry, IdOrName, NamedImageResourceDirectoryEntry,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IdOrNamedImageResourceDirectoryEntry {
    Id(IdImageResourceDirectoryEntry),
    Named(NamedImageResourceDirectoryEntry),
}

impl IdOrNamedImageResourceDirectoryEntry {
    pub fn into_id(self) -> Option<IdImageResourceDirectoryEntry> {
        match self {
            Self::Id(id) => Some(id),
            Self::Named(named) => None,
        }
    }

    pub const fn id(&self) -> u32 {
        match self {
            Self::Id(id) => id.id(),
            Self::Named(named) => named.raw_id(),
        }
    }

    pub fn id_or_name(&self) -> IdOrName {
        match self {
            Self::Id(id) => IdOrName::Id(id.id()),
            Self::Named(named) => IdOrName::Name(named.name().to_owned()),
        }
    }

    /// Returns the data associated to this directory entry.
    pub fn data<R>(
        self,
        section: &mut ResourceDirectory<R>,
    ) -> io::Result<ResourceDirectoryEntryData>
    where
        R: Read + Seek,
    {
        match self {
            Self::Id(id) => id.data(section),
            Self::Named(named) => named.data(section),
        }
    }
}
