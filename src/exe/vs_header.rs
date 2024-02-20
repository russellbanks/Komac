use crate::exe::utils::get_widestring_size;
use color_eyre::eyre::Result;
use object::ReadRef;
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
    pub fn parse<R: ReadRef<'data>>(data: R, base_offset: u64) -> Result<(u64, Self)> {
        let mut offset = base_offset;

        let length = data.read(&mut offset).unwrap();

        let value_length = data.read(&mut offset).unwrap();

        let type_value = data.read(&mut offset).unwrap();

        let widestring_size = get_widestring_size(data, offset);

        let key = data
            .read_slice(&mut offset, usize::try_from(widestring_size)?)
            .unwrap();
        offset += mem::size_of::<u16>() as u64;

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
