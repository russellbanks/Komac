use std::{
    fmt, io,
    io::{Read, Seek},
};

use super::file_entry::FileEntry;

#[derive(Clone)]
pub struct NamedFileEntry {
    file_entry: FileEntry,
    name: String,
}

impl NamedFileEntry {
    /// Creates a new [`NamedFileEntry] from a [`FileEntry`] and a name.
    pub fn new<S>(file_entry: FileEntry, name: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            file_entry,
            name: name.into(),
        }
    }

    #[inline]
    const fn r#type(&self) -> [u32; 2] {
        self.file_entry.r#type()
    }

    #[inline]
    pub const fn xor_flag(&self) -> u32 {
        self.file_entry.xor_flag()
    }

    #[inline]
    pub const fn size(&self) -> u32 {
        self.file_entry.size()
    }

    #[inline]
    pub const fn offset(&self) -> u32 {
        self.file_entry.offset()
    }

    /// Returns the size of the name in UTF-16LE characters.
    #[inline]
    pub const fn name_size(&self) -> u32 {
        self.file_entry.name_size()
    }

    #[inline]
    pub const fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn read_file<R>(&self, reader: &mut R) -> io::Result<Vec<u8>>
    where
        R: Read + Seek,
    {
        self.file_entry.read_file(reader)
    }
}

impl fmt::Debug for NamedFileEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NamedFileEntry")
            .field("type", &self.r#type())
            .field("xor_flag", &self.xor_flag())
            .field("size", &self.size())
            .field("offset", &self.offset())
            .field("name_size", &self.name_size())
            .field("name", &self.name())
            .finish()
    }
}
