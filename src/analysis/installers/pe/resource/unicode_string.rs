use std::io;

use zerocopy::{IntoBytes, LE, U16};

use crate::read::ReadBytesExt;

pub struct UnicodeString {
    length: u16,
    buf: Vec<U16<LE>>,
}

impl UnicodeString {
    pub fn read_from<R>(mut reader: R) -> io::Result<Self>
    where
        R: io::Read + io::Seek,
    {
        let length = reader.read_u16::<LE>()?;
        let mut buf = vec![U16::ZERO; length.into()];
        reader.read_exact(buf.as_mut_bytes())?;

        Ok(Self { length, buf })
    }

    pub fn to_string_lossy(self) -> String {
        let d = self.buf.into_iter().map(U16::get);

        char::decode_utf16(d)
            .map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER))
            .collect::<String>()
    }
}
