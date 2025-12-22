use std::{
    io,
    io::{Read, Seek},
};

use super::{
    ImageResourceDirectoryEntry, NamedImageResourceDirectoryEntry, ResourceDirectoryTable,
    ResourceType, SectionReader,
};

pub struct ResourceDirectory<R: Read + Seek> {
    reader: SectionReader<R>,
    current_directory_table: ResourceDirectoryTable,
}

impl<R: Read + Seek> ResourceDirectory<R> {
    pub fn new(mut reader: SectionReader<R>) -> io::Result<Self> {
        ResourceDirectoryTable::read_from(&mut reader).map(|root_directory_table| Self {
            reader,
            current_directory_table: root_directory_table,
        })
    }

    #[inline]
    pub const fn reader_mut(&mut self) -> &mut SectionReader<R> {
        &mut self.reader
    }

    pub const fn current_directory_table(&self) -> &ResourceDirectoryTable {
        &self.current_directory_table
    }

    pub fn navigate_to_rc_data(&mut self) -> io::Result<&ResourceDirectoryTable> {
        self.navigate_to_directory_table(ResourceType::RCData)
    }

    pub fn navigate_to_manifest(&mut self) -> io::Result<&ResourceDirectoryTable> {
        self.navigate_to_directory_table(ResourceType::Manifest)
    }

    pub fn navigate_to_version_info(&mut self) -> io::Result<&ResourceDirectoryTable> {
        self.navigate_to_directory_table(ResourceType::Version)
    }

    pub fn navigate_to_directory_table(
        &mut self,
        resource_type: ResourceType,
    ) -> io::Result<&ResourceDirectoryTable> {
        self.navigate_to_directory_table_by_id(resource_type.id())
    }

    pub fn navigate_to_directory_table_by_id(
        &mut self,
        id: u32,
    ) -> io::Result<&ResourceDirectoryTable> {
        let mut directory_entry =
            self.current_directory_table
                .find_id_entry(id)
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("{id} not found in current directory table"),
                    )
                })?;

        self.current_directory_table = directory_entry.data(self)?.table().unwrap();
        Ok(&self.current_directory_table)
    }

    pub fn navigate_to_directory_table_by_name(
        &mut self,
        name: &str,
    ) -> io::Result<&ResourceDirectoryTable> {
        let mut directory_entry = self
            .current_directory_table
            .find_name_entry(name)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("{name} not found in current directory table"),
                )
            })?;

        self.current_directory_table = directory_entry.entry().data(self)?.table().unwrap();
        Ok(&self.current_directory_table)
    }
}
