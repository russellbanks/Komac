use byteorder::{ReadBytesExt, LE};
use encoding_rs::Encoding;
use std::io::{Read, Result};

/// Read an encoded String where the length is stored in the 4 bytes immediately prior
pub fn encoded_string<R: Read>(
    reader: &mut R,
    encoding: &'static Encoding,
) -> Result<Option<String>> {
    let length = reader.read_u32::<LE>()?;
    if length == 0 {
        return Ok(None);
    }
    let mut buf = vec![0; length as usize];
    reader.read_exact(&mut buf)?;
    Ok(Some(encoding.decode(&buf).0.into_owned()))
}

/// Read an encoded String where the length is known
pub fn sized_encoded_string<R: Read>(
    reader: &mut R,
    length: u32,
    encoding: &'static Encoding,
) -> Result<Option<String>> {
    if length == 0 {
        return Ok(None);
    }
    let mut buf = vec![0; length as usize];
    reader.read_exact(&mut buf)?;
    Ok(Some(encoding.decode(&buf).0.into_owned()))
}
