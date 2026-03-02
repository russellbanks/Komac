use std::{fmt, io};

use indexmap::IndexMap;
use zerocopy::FromBytes;

use super::{VSHeader, VSStringTable, VSType};

/// Represents a [`StringFileInfo`](https://docs.microsoft.com/windows/win32/menurc/stringfileinfo) structure.
#[derive(Clone)]
pub struct VSStringFileInfo<'a> {
    header: VSHeader<'a>,
    children: Vec<VSStringTable<'a>>,
}

impl<'a> VSStringFileInfo<'a> {
    pub fn read_from(data: &'a [u8]) -> io::Result<Self> {
        let header = VSHeader::read_from(data)?;

        let mut children = Vec::new();

        let mut offset = header.end_offset;

        while offset < usize::from(header.length()) {
            let child = VSStringTable::read_from(&data[offset..])?;

            offset += usize::from(child.length());
            offset = offset.next_multiple_of(size_of::<u32>());
            children.push(child);
        }

        debug_assert!(!children.is_empty());

        Ok(Self { header, children })
    }

    /// The length, in bytes, of the entire StringFileInfo block, including all structures indicated
    /// by the Children member.
    #[must_use]
    #[inline]
    pub const fn length(&self) -> u16 {
        self.header.length()
    }

    /// This member is always equal to zero.
    #[must_use]
    #[inline]
    pub const fn value_length(&self) -> u16 {
        self.header.value_length()
    }

    /// The type of data in the version resource.
    ///
    /// This member is 1 if the version resource contains text data and 0 if the version resource
    /// contains binary data.
    #[must_use]
    #[inline]
    const fn r#type(&self) -> VSType {
        self.header.r#type()
    }

    /// The Unicode string "StringFileInfo".
    #[must_use]
    #[inline]
    pub fn key(&self) -> &str {
        self.header.key()
    }

    /// An array of one or more [`StringTable`] structures.
    ///
    /// Each **StringTable** structure's **szKey** member indicates the appropriate language and
    /// code page for displaying the text in that **StringTable** structure.
    ///
    /// [`StringTable`]: VSStringTable
    #[must_use]
    #[inline]
    pub const fn children(&self) -> &[VSStringTable<'_>] {
        self.children.as_slice()
    }

    pub fn entries(&'a self) -> impl Iterator<Item = (&'a str, &'a str)> {
        self.children.first().map(VSStringTable::entries).unwrap()
    }

    pub fn table(&'a self) -> IndexMap<&'a str, &'a str> {
        self.children.first().map(VSStringTable::table).unwrap()
    }
}

impl fmt::Debug for VSStringFileInfo<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("StringFileInfo")
            .field("wLength", &self.length())
            .field("wValueLength", &self.value_length())
            .field("wType", &self.r#type())
            .field("szKey", &self.key())
            .field("Children", &self.children())
            .finish()
    }
}
