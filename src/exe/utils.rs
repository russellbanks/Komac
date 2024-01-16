use byteorder::{LittleEndian, ReadBytesExt};
use color_eyre::eyre::Result;
use std::io::Cursor;
use std::mem;

pub const fn align(value: usize, boundary: usize) -> usize {
    if value % boundary == 0 {
        value
    } else {
        value + (boundary - (value % boundary))
    }
}

pub fn get_widestring_size(data: &[u8], offset: usize) -> Result<usize> {
    let mut cursor = Cursor::new(data);
    let mut index = offset;
    for i in (index..data.len()).step_by(mem::size_of::<u16>()) {
        cursor.set_position(i as u64);

        if cursor.read_u16::<LittleEndian>()? == 0 {
            index = i;
            break;
        }
    }

    Ok(index - offset)
}
