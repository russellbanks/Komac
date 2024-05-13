use color_eyre::eyre::{eyre, Result};
use object::ReadRef;

use crate::exe::utils::{align, get_widestring_size};
use crate::exe::vs_header::VSHeader;

/// Represents a [`String`](https://docs.microsoft.com/en-us/windows/win32/menurc/string-str) structure.
pub struct VSString<'data> {
    pub header: VSHeader<'data>,
    pub value: &'data [u16],
}
impl<'data> VSString<'data> {
    /// Parse a `VSString` object at the given virtual address.
    pub fn parse<R: ReadRef<'data>>(data: R, base_offset: u64) -> Result<Self> {
        let (mut offset, header) = VSHeader::parse(data, base_offset)?;
        offset = align(offset, 4);

        let widestring_size = get_widestring_size(data, offset);
        let value = data
            .read_slice_at(offset, usize::try_from(widestring_size)?)
            .map_err(|_| {
                eyre!("Failed to read widestring slice of length {widestring_size} at offset {offset}")
            })?;

        Ok(Self { header, value })
    }
}
