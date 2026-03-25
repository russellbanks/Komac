mod directory;
mod directory_entry_data;
mod directory_table;
mod image;
mod iterator;
mod name;
mod resource_types;
mod section_reader;

use std::{
    fmt,
    io::{Read, Seek},
};

pub use directory::ResourceDirectory;
pub use directory_entry_data::ResourceDirectoryEntryData;
pub use directory_table::ResourceDirectoryTable;
pub use image::{
    IdImageResourceDirectoryEntry, IdOrName, IdOrNamedImageResourceDirectoryEntry,
    ImageResourceDataEntry, ImageResourceDirectory, ImageResourceDirectoryEntry,
    NamedImageResourceDirectoryEntry,
};
pub use iterator::ResourceIter;
pub use resource_types::ResourceType;
pub use section_reader::SectionReader;
use zerocopy::{FromBytes, FromZeros, IntoBytes};

pub struct Resource {
    r#type: IdOrName,
    name_id: u32,
    language_id: u32,
    entry: ImageResourceDataEntry,
}

impl Resource {
    pub const fn id_or_name(&self) -> &IdOrName {
        &self.r#type
    }
    pub const fn name(&self) -> Option<&str> {
        match self.r#type {
            IdOrName::Name(ref name) => Some(name.as_str()),
            IdOrName::Id(_) => None,
        }
    }

    #[inline]
    pub const fn offset_to_data(&self) -> u32 {
        self.entry.offset_to_data()
    }

    #[inline]
    pub const fn size(&self) -> u32 {
        self.entry.size()
    }
}

impl fmt::Debug for Resource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buffer = itoa::Buffer::new();

        f.debug_struct("Resource")
            .field(
                "TypeID",
                &match self.r#type {
                    IdOrName::Id(id) => ResourceType::try_from(id)
                        .map(ResourceType::as_str)
                        .unwrap_or_else(|_| buffer.format(id)),
                    IdOrName::Name(ref name) => name,
                },
            )
            .field("NameID", &self.name_id)
            .field("LanguageID", &self.language_id)
            .field("Entry", &self.entry)
            .finish()
    }
}
