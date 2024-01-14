use crate::exe::utils::get_widestring_size;
use color_eyre::eyre::Result;
use object::ReadRef;
use std::mem;
use zerocopy::FromBytes;

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

        let length = data.read_at(offset as u64).unwrap();
        offset += mem::size_of::<u16>();

        let value_length = data.read_at(offset as u64).unwrap();
        offset += mem::size_of::<u16>();

        let type_value = data.read_at(offset as u64).unwrap();
        offset += mem::size_of::<u16>();

        let widestring_size = get_widestring_size(data, offset)?;

        let key = FromBytes::slice_from(&data[offset..offset + widestring_size]).unwrap();
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
