use bitflags::bitflags;
use color_eyre::eyre::{ensure, eyre, Result};
use zerocopy::little_endian::U32;
use zerocopy::{FromBytes, FromZeroes};

#[derive(Debug, FromBytes, FromZeroes)]
#[repr(transparent)]
struct HeaderFlags(u32);

bitflags! {
    impl HeaderFlags: u32 {
        const UNINSTALL = 1 << 0;
        const SILENT = 1 << 1;
        const NO_CRC = 1 << 2;
        const FORCE_CRC = 1 << 3;
        // NSISBI fork flags:
        const BI_LONG_OFFSET = 1 << 4;
        const BI_EXTERNAL_FILE_SUPPORT = 1 << 5;
        const BI_EXTERNAL_FILE = 1 << 6;
        const BI_IS_STUB_INSTALLER = 1 << 7;
    }
}

#[derive(Debug, FromBytes, FromZeroes)]
#[repr(C)]
pub struct FirstHeader {
    flags: HeaderFlags,
    signature: U32,
    magic: [u8; FirstHeader::MAGIC_SIZE as usize],
    pub header_size: U32,
    length_of_following_data: U32,
}

impl FirstHeader {
    /// The NSIS first header is aligned to 512 bytes
    pub const ALIGNMENT: u16 = 512;

    /// Signature that appears directly before the NSIS magic bytes, `NullsoftInst`
    const SIGNATURE: u32 = 0xDEAD_BEEF;

    const MAGIC_SIZE: u8 = 12;

    const MAGIC: &'static [u8; Self::MAGIC_SIZE as usize] = b"NullsoftInst";

    pub fn read(data: &[u8]) -> Result<&Self> {
        let first_header = Self::ref_from(&data[..size_of::<Self>()]).unwrap();
        if first_header.signature.get() == Self::SIGNATURE && &first_header.magic == Self::MAGIC {
            ensure!(first_header.length_of_following_data.get() as usize > size_of::<Self>());
            return Ok(first_header);
        }

        Err(eyre!("No NSIS first header found"))
    }
}
