use byteorder::{ReadBytesExt, LE};
use encoding_rs::Encoding;
use std::io::{Read, Result};

#[derive(Debug, Default)]
pub struct InnoValue(Vec<u8>);

impl InnoValue {
    pub fn new_raw<R: Read>(reader: &mut R) -> Result<Option<Vec<u8>>> {
        let length = reader.read_u32::<LE>()?;
        if length == 0 {
            return Ok(None);
        }
        let mut buf = vec![0; length as usize];
        reader.read_exact(&mut buf)?;
        Ok(Some(buf))
    }

    pub fn new_encoded<R: Read>(reader: &mut R) -> Result<Option<Self>> {
        Self::new_raw(reader).map(|opt_bytes| opt_bytes.map(Self))
    }

    pub fn new_string<R: Read>(
        reader: &mut R,
        codepage: &'static Encoding,
    ) -> Result<Option<String>> {
        Self::new_encoded(reader)
            .map(|opt_value| opt_value.map(|value| value.into_string(codepage)))
    }

    pub fn new_sized_string<R: Read>(
        reader: &mut R,
        length: u32,
        codepage: &'static Encoding,
    ) -> Result<Option<String>> {
        if length == 0 {
            return Ok(None);
        }
        let mut buf = vec![0; length as usize];
        reader.read_exact(&mut buf)?;
        Ok(Some(codepage.decode(&buf).0.into_owned()))
    }

    pub fn into_string(self, codepage: &'static Encoding) -> String {
        codepage.decode(&self.0).0.into_owned()
    }
}
