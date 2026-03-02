use std::{fmt, io};

use encoding_rs::Encoding;
use indexmap::IndexMap;
use zerocopy::FromBytes;

use super::{VSHeader, VSString};

#[derive(Clone)]
pub struct VSStringTable<'a> {
    header: VSHeader<'a>,
    children: Vec<VSString<'a>>,
}

impl<'a> VSStringTable<'a> {
    pub fn read_from(data: &'a [u8]) -> io::Result<Self> {
        let header = VSHeader::read_from(data)?;

        let mut children = Vec::new();

        let mut offset = header.end_offset;

        while offset < usize::from(header.length()) {
            let child = VSString::read_from(&data[offset..])?;

            offset += usize::from(child.length());
            offset = offset.next_multiple_of(size_of::<u32>());
            children.push(child);
        }

        Ok(Self { header, children })
    }

    // The length, in bytes, of this StringTable structure, including all structures indicated by
    // the Children member.
    #[must_use]
    #[inline]
    pub const fn length(&self) -> u16 {
        self.header.length()
    }

    /// Returns the value length, which is always equal to zero in a `StringTable`.
    #[must_use]
    #[inline]
    pub const fn value_length(&self) -> u16 {
        self.header.value_length()
    }

    /// Returns the raw codepage value.
    #[must_use]
    #[inline]
    pub fn raw_codepage(&self) -> u16 {
        self.header.string_table_raw_codepage()
    }

    pub fn codepage(&self) -> &'static Encoding {
        self.header.string_table_codepage()
    }

    /// Returns the Children as a slice of one or more [`String`] structures.
    ///
    /// [`String`]: VSString
    #[must_use]
    #[inline]
    pub const fn children(&self) -> &[VSString<'a>] {
        self.children.as_slice()
    }

    pub fn entries(&'a self) -> impl Iterator<Item = (&'a str, &'a str)> {
        self.children
            .iter()
            .map(|vs_string| (vs_string.key(), vs_string.value()))
    }

    pub fn table(&'a self) -> IndexMap<&'a str, &'a str> {
        self.entries().collect::<IndexMap<_, _>>()
    }
}

impl fmt::Debug for VSStringTable<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StringTable")
            .field("wLength", &self.header.length())
            .field("wValueLength", &self.header.value_length())
            .field("wType", &self.header.r#type())
            .field("szKey", &self.header.key())
            .field("RawCodepage", &self.raw_codepage())
            .field("Codepage", &self.codepage())
            .field("Children", &self.children())
            .field("Table", &self.table())
            .finish()
    }
}
