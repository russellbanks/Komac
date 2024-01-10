use byteorder::{LittleEndian, ReadBytesExt};
use color_eyre::eyre::Result;
use std::io::Cursor;
use std::mem;

/// Represents a header for a `VS_VERSION` structure.
///
/// This is not an officially documented header, but rather is added to make parsing these aspects of the structures
/// a little bit easier.
pub struct VSHeader<'data> {
    pub length: &'data u16,
    pub value_length: &'data u16,
    pub type_: &'data u16,
    pub key: &'data [u16],
}

impl<'data> VSHeader<'data> {
    pub fn parse(data: &'data [u8], base_offset: usize) -> Result<(usize, Self)> {
        let mut offset = base_offset;

        let length = bytemuck::from_bytes::<u16>(&data[offset..offset + mem::size_of::<u16>()]);
        offset += mem::size_of::<u16>();

        let value_length =
            bytemuck::from_bytes::<u16>(&data[offset..offset + mem::size_of::<u16>()]);
        offset += mem::size_of::<u16>();

        let type_value = bytemuck::from_bytes::<u16>(&data[offset..offset + mem::size_of::<u16>()]);
        offset += mem::size_of::<u16>();

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

        let key = bytemuck::cast_slice(&data[offset..offset + widestring_size]);
        let key_size = (key.len() + 1) * mem::size_of::<u16>();
        offset += key_size;

        Ok((
            offset,
            VSHeader {
                length,
                value_length,
                type_: type_value,
                key,
            },
        ))
    }
}
