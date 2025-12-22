mod directory;
mod directory_entry_data;
mod image;
mod name;
mod resource_types;
mod section_reader;

use std::{
    io,
    io::{Read, Seek},
};

pub use directory::ResourceDirectory;
pub use image::{
    ImageResourceDataEntry, ImageResourceDirectory, ImageResourceDirectoryEntry,
    NamedImageResourceDirectoryEntry,
};
pub use resource_types::ResourceType;
pub use section_reader::SectionReader;
use zerocopy::{FromBytes, FromZeros, IntoBytes};

#[derive(Clone, Debug)]
pub struct ResourceDirectoryTable {
    pub header: ImageResourceDirectory,
    name_entries: Vec<NamedImageResourceDirectoryEntry>,
    id_entries: Vec<ImageResourceDirectoryEntry>,
}

impl ResourceDirectoryTable {
    pub fn read_from<R>(src: &mut SectionReader<R>) -> io::Result<Self>
    where
        R: Read + Seek,
    {
        let header = ImageResourceDirectory::read_from_io(&mut *src)?;

        let mut name_entries =
            vec![ImageResourceDirectoryEntry::new_zeroed(); header.number_of_name_entries().into()];

        for name_entry in &mut name_entries {
            src.read_exact(name_entry.as_mut_bytes())?;
        }

        let mut id_entries =
            vec![ImageResourceDirectoryEntry::new_zeroed(); header.number_of_id_entries().into()];

        for id_entry in &mut id_entries {
            src.read_exact(id_entry.as_mut_bytes())?;
        }

        Ok(Self {
            header,
            name_entries: name_entries
                .into_iter()
                .flat_map(|entry| {
                    entry
                        .name()
                        .to_string_lossy(src)
                        .map(|name| NamedImageResourceDirectoryEntry::new(name, entry))
                })
                .collect(),
            id_entries,
        })
    }

    pub fn find_id_entry(&self, id: u32) -> Option<&ImageResourceDirectoryEntry> {
        self.id_entries().find(|entry| entry.name_or_id() == id)
    }

    pub fn find_name_entry(&self, name: &str) -> Option<&NamedImageResourceDirectoryEntry> {
        self.name_entries().find(|entry| entry.name() == name)
    }

    #[inline]
    pub fn id_entries(&self) -> impl Iterator<Item = &ImageResourceDirectoryEntry> {
        self.id_entries.iter()
    }

    #[inline]
    pub fn name_entries(&self) -> impl Iterator<Item = &NamedImageResourceDirectoryEntry> {
        self.name_entries.iter()
    }
}
