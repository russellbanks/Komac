use std::{
    io,
    io::{Read, Seek, SeekFrom},
};

use super::SectionReader;
use crate::analysis::installers::pe::resource::unicode_string::UnicodeString;

/// A resource name.
#[derive(Debug, Clone, Copy)]
pub struct ResourceName {
    offset: u32,
}

impl ResourceName {
    #[inline]
    pub fn new(offset: u32) -> Self {
        Self { offset }
    }

    /// Converts to a `String`.
    pub fn to_string_lossy<R: Read + Seek>(
        &self,
        reader: &mut SectionReader<R>,
    ) -> io::Result<String> {
        reader.seek(SeekFrom::Start(self.offset.into()))?;
        UnicodeString::read_from(reader).map(UnicodeString::to_string_lossy)
    }
}
