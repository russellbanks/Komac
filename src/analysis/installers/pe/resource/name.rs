use std::{
    io,
    io::{Read, Seek, SeekFrom},
};

use encoding_rs::UTF_16LE;
use zerocopy::{IntoBytes, LE};

use super::SectionReader;
use crate::read::ReadBytesExt;

/// A resource name.
#[derive(Debug, Clone, Copy)]
pub struct ResourceName {
    offset: u32,
}

impl ResourceName {
    #[inline]
    pub const fn new(offset: u32) -> Self {
        Self { offset }
    }

    /// Converts to a `String`, replacing invalid data with
    /// [the replacement character (`U+FFFD`)][U+FFFD].
    ///
    /// [U+FFFD]: core::char::REPLACEMENT_CHARACTER
    pub fn to_string_lossy<R: Read + Seek>(
        self,
        reader: &mut SectionReader<R>,
    ) -> io::Result<String> {
        reader.seek(SeekFrom::Start(self.offset.into()))?;

        let length = reader.read_u16::<LE>()?;
        let mut buf = vec![0; usize::from(length) * size_of::<u16>()];
        reader.read_exact(buf.as_mut_bytes())?;

        /// Could be replaced with [`String::from_utf16le_lossy`]
        Ok(UTF_16LE.decode_without_bom_handling(&buf).0.into_owned())
    }
}
