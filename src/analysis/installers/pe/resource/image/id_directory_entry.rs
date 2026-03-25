use std::{
    fmt, io,
    io::{Read, Seek},
};

use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, LittleEndian, U32};

use crate::analysis::installers::pe::resource::{
    ImageResourceDataEntry, ImageResourceDirectoryEntry, ResourceDirectory, ResourceDirectoryTable,
    ResourceType, directory_entry_data::ResourceDirectoryEntryData,
};

#[derive(Copy, Clone, Eq, PartialEq, FromBytes, IntoBytes, Immutable, KnownLayout)]
#[repr(transparent)]
pub struct IdImageResourceDirectoryEntry(ImageResourceDirectoryEntry);

impl IdImageResourceDirectoryEntry {
    #[inline]
    pub const fn new(entry: ImageResourceDirectoryEntry) -> Self {
        Self(entry)
    }

    /// Returns a 32-bit integer that identifies the Type, Name, or Language ID entry.
    #[inline]
    pub const fn id(self) -> u32 {
        self.entry().name_or_id()
    }

    #[inline]
    pub const fn entry(self) -> ImageResourceDirectoryEntry {
        self.0
    }

    /// Returns the data associated to this directory entry.
    pub fn data<R>(
        self,
        section: &mut ResourceDirectory<R>,
    ) -> io::Result<ResourceDirectoryEntryData>
    where
        R: Read + Seek,
    {
        self.entry().data(section)
    }

    /// Returns the section offset of the associated table or data.
    #[inline]
    pub const fn data_offset(self) -> u32 {
        self.entry().data_offset()
    }

    #[inline]
    pub const fn file_offset(self, resource_offset: u32) -> u32 {
        self.entry().file_offset(resource_offset)
    }
}

impl fmt::Debug for IdImageResourceDirectoryEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut buffer = itoa::Buffer::new();

        f.debug_struct("IdImageResourceDirectoryEntry")
            .field(
                "ID",
                &ResourceType::try_from(self.id())
                    .map(ResourceType::as_str)
                    .unwrap_or_else(|_| buffer.format(self.id())),
            )
            .field("Table", &self.entry().is_table())
            .field("Offset", &self.data_offset())
            .finish()
    }
}
