mod flags;
mod signature;

use std::{fmt, io};

use flags::HeaderFlags;
use signature::NsisSignature;
use zerocopy::{Immutable, KnownLayout, LE, TryFromBytes, U32};

/// Represents the first header of an NSIS installer.
///
/// This struct corresponds to the `firstheader` structure in the NSIS source code:
/// <https://github.com/NSIS-Dev/nsis/blob/v311/Source/exehead/fileform.h#L246>
///
/// The `FirstHeader` contains magic signatures, flags, and size information for the installer data.
///
/// # Layout
///
/// | Offset | Field                          | Size | Description                                                        |
/// |--------|--------------------------------|------|--------------------------------------------------------------------|
/// | 0x00   | `flags`                        | 4    | Installer flags (FH_FLAGS_*)                                       |
/// | 0x04   | `siginfo`                      | 4    | Magic number 0xDEADBEEF (FH_SIG)                                   |
/// | 0x08   | `nsinst`                       | 12   | "NullsoftInst" signature                                           |
/// | 0x14   | `length_of_header`             | 4    | Size of this header                                                |
/// | 0x18   | `length_of_all_following_data` | 4    | Size of all remaining installer data including this header and CRC |
#[derive(Copy, Clone, TryFromBytes, KnownLayout, Immutable)]
#[repr(C, packed)]
pub struct FirstHeader {
    flags: HeaderFlags,
    signature: NsisSignature,
    length_of_header: U32<LE>,

    /// The length of all the data (including the [`FirstHeader`] and CRC).
    length_of_following_data: U32<LE>,
}

impl FirstHeader {
    pub fn try_read_from_io<R>(mut src: R) -> io::Result<Self>
    where
        Self: Sized,
        R: io::Read,
    {
        let mut buf = [0; size_of::<Self>()];
        src.read_exact(&mut buf)?;
        Self::try_read_from_bytes(&buf)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.to_string()))
    }

    #[inline]
    pub const fn flags(&self) -> HeaderFlags {
        self.flags
    }

    #[inline]
    pub const fn signature(&self) -> NsisSignature {
        self.signature
    }

    #[inline]
    pub const fn length_of_header(&self) -> u32 {
        self.length_of_header.get()
    }

    /// Returns the length of all the data (including the [`FirstHeader`] and CRC).
    #[inline]
    pub const fn length_of_following_data(&self) -> u32 {
        self.length_of_following_data.get()
    }
}

impl fmt::Debug for FirstHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("firstheader")
            .field("flags", &self.flags())
            .field("nsinst", &self.signature())
            .field("length_of_header", &self.length_of_header())
            .field(
                "length_of_all_following_data",
                &self.length_of_following_data(),
            )
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use zerocopy::{ValidityError, try_transmute};

    use super::{FirstHeader, HeaderFlags};

    #[test]
    fn first_header_size() {
        const EXPECTED_FIRST_HEADER_SIZE: usize = size_of::<u32>() * 7;

        assert_eq!(size_of::<FirstHeader>(), EXPECTED_FIRST_HEADER_SIZE);
    }

    #[test]
    fn first_header_from_bytes()
    -> Result<(), ValidityError<[u8; size_of::<FirstHeader>()], FirstHeader>> {
        /// The `firstheader` bytes of BetterDiscord.
        const FIRST_HEADER_BYTES: [u8; size_of::<FirstHeader>()] = [
            0x04, 0x00, 0x00, 0x00, 0xEF, 0xBE, 0xAD, 0xDE, 0x4E, 0x75, 0x6C, 0x6C, 0x73, 0x6F,
            0x66, 0x74, 0x49, 0x6E, 0x73, 0x74, 0x1C, 0x89, 0x00, 0x00, 0xC6, 0x52, 0xAE, 0x04,
        ];

        let header: FirstHeader = try_transmute!(FIRST_HEADER_BYTES)?;

        assert_eq!(header.length_of_header(), 35_100);
        assert_eq!(header.length_of_following_data(), 78_533_318);
        assert_eq!(header.flags(), HeaderFlags::NO_CRC);

        Ok(())
    }
}
