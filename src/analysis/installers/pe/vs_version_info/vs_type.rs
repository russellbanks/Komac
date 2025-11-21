use std::{io, io::Read};

use zerocopy::{Immutable, KnownLayout, TryFromBytes};

#[derive(Copy, Clone, Debug, Eq, PartialEq, TryFromBytes, KnownLayout, Immutable)]
#[repr(u16)]
pub enum VSType {
    Binary = 0,
    String = 1_u16.to_le(),
}

impl VSType {
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

    /// Returns `true` if self is Binary.
    #[must_use]
    #[inline]
    pub const fn is_binary(self) -> bool {
        matches!(self, Self::Binary)
    }

    /// Returns `true` if self is String.
    #[must_use]
    #[inline]
    pub const fn is_string(self) -> bool {
        matches!(self, Self::String)
    }
}
