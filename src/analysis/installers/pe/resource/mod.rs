mod directory;
mod directory_entry_data;
mod image;
mod name;
mod resource_types;
mod section_reader;
mod unicode_string;

use std::io;

pub use directory::ResourceDirectory;
pub use image::{ImageResourceDataEntry, ImageResourceDirectory, ImageResourceDirectoryEntry};
pub use resource_types::ResourceType;
pub use section_reader::SectionReader;
use zerocopy::{FromBytes, FromZeros, IntoBytes};

#[derive(Clone, Debug)]
pub struct ResourceDirectoryTable {
    pub header: ImageResourceDirectory,
    name_entries: Vec<ImageResourceDirectoryEntry>,
    id_entries: Vec<ImageResourceDirectoryEntry>,
}

impl ResourceDirectoryTable {
    pub fn read_from<R>(mut src: R) -> io::Result<Self>
    where
        R: io::Read,
    {
        let header = ImageResourceDirectory::read_from_io(&mut src)?;

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
            name_entries,
            id_entries,
        })
    }

    pub fn find_id_entry(&self, id: u32) -> Option<&ImageResourceDirectoryEntry> {
        self.id_entries().find(|entry| entry.name_or_id() == id)
    }

    pub fn find_name_entry<R>(
        &self,
        section_reader: &mut SectionReader<R>,
        name: &str,
    ) -> Option<&ImageResourceDirectoryEntry>
    where
        R: io::Read + io::Seek,
    {
        self.name_entries().find(|entry| {
            entry
                .name()
                .to_string_lossy(section_reader)
                .inspect(|x| {
                    dbg!(&x);
                })
                .is_ok_and(|entry_name| entry_name == name)
        })
    }

    #[inline]
    pub fn id_entries(&self) -> impl Iterator<Item = &ImageResourceDirectoryEntry> {
        self.id_entries.iter()
    }

    #[inline]
    pub fn name_entries(&self) -> impl Iterator<Item = &ImageResourceDirectoryEntry> {
        self.name_entries.iter()
    }

    pub fn entries(&self) -> impl Iterator<Item = &ImageResourceDirectoryEntry> {
        self.name_entries().chain(self.id_entries())
    }
}
