use crate::exe::utils::{align, get_widestring_size};
use crate::exe::vs_header::VSHeader;
use color_eyre::eyre::Result;
use object::ReadRef;

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
            .unwrap();

        Ok(Self { header, value })
    }
}
