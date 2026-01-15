use std::{fmt, io, io::Read};

use zerocopy::{Immutable, IntoBytes, KnownLayout, TryFromBytes};

/// PE Magic: PE\0\0, little endian
#[derive(Copy, Clone, Default, Eq, PartialEq, TryFromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(u32)]
pub enum Signature {
    #[default]
    Magic = u32::from_le_bytes(*b"PE\0\0"),
}

impl Signature {
    pub fn try_read_from<R>(mut reader: R) -> io::Result<Self>
    where
        Self: Sized,
        R: Read,
    {
        let mut buf = [0; size_of::<Self>()];
        reader.read_exact(&mut buf)?;
        Self::try_read_from_bytes(&buf)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.to_string()))
    }

    pub fn as_str(&self) -> &str {
        std::str::from_utf8(self.as_bytes()).unwrap_or_default()
    }
}

impl fmt::Display for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Debug for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Signature").field(&self.as_str()).finish()
    }
}
