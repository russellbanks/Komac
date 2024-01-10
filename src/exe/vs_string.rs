use crate::exe::utils::align;
use crate::exe::vs_header::VSHeader;
use byteorder::{LittleEndian, ReadBytesExt};
use color_eyre::eyre::Result;
use std::io::Cursor;

/// Represents a [`String`](https://docs.microsoft.com/en-us/windows/win32/menurc/string-str) structure.
pub struct VSString<'data> {
    pub header: VSHeader<'data>,
    pub value: &'data [u16],
}
impl<'data> VSString<'data> {
    /// Parse a `VSString` object at the given [`RVA`](RVA).
    pub fn parse(data: &'data [u8], base_offset: usize) -> Result<Self> {
        let (mut offset, header) = VSHeader::parse(data, base_offset)?;
        offset = align(offset, 4);

        let widestring_size = {
            let mut cursor = Cursor::new(data);
            let mut index = offset;
            for i in (index..data.len()).step_by(2) {
                cursor.set_position(i as u64);

                if cursor.read_u16::<LittleEndian>()? == 0 {
                    index = i;
                    break;
                }
            }

            index - offset
        };
        let value = bytemuck::cast_slice(&data[offset..offset + widestring_size]);

        Ok(Self { header, value })
    }
}
