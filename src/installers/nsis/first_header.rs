use bitflags::bitflags;
use byteorder::{ReadBytesExt, LE};
use color_eyre::eyre::{ensure, eyre, Result};
use std::io::{Read, Seek, SeekFrom};

bitflags! {
    #[derive(Copy, Clone, Debug)]
    struct HeaderFlags: u32 {
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

#[expect(dead_code)]
#[derive(Debug)]
pub struct FirstHeader {
    flags: HeaderFlags,
    pub header_size: u32,
    arc_size: u32,
    header_offset: u64,
    pub data_offset: u64,
}

impl FirstHeader {
    const SIZE: u8 = 4 * 7;

    /// The NSIS header is aligned to 512 bytes
    const ALIGNMENT: u16 = 512;

    /// Signature that appears directly before the NSIS magic bytes, `NullsoftInst`
    const SIGNATURE: u32 = 0xDEAD_BEEF;

    const MAGIC_SIZE: u8 = 12;

    const MAGIC: &'static [u8; Self::MAGIC_SIZE as usize] = b"NullsoftInst";

    const MAX_HEADER_POSITION: u64 = 1 << 20;

    pub fn read<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        let mut pos = 0u64;

        while pos < Self::MAX_HEADER_POSITION {
            let flags = HeaderFlags::from_bits_retain(reader.read_u32::<LE>()?);
            let signature = reader.read_u32::<LE>()?;
            let mut magic = [0; Self::MAGIC_SIZE as usize];
            reader.read_exact(&mut magic)?;

            if signature == Self::SIGNATURE && &magic == Self::MAGIC {
                let header_size = reader.read_u32::<LE>()?;
                let arc_size = reader.read_u32::<LE>()?;
                ensure!(arc_size > u32::from(Self::SIZE));
                return Ok(Self {
                    flags,
                    header_size,
                    arc_size,
                    header_offset: pos,
                    data_offset: pos + u64::from(Self::SIZE),
                });
            }

            pos += u64::from(Self::ALIGNMENT);

            reader.seek(SeekFrom::Start(pos))?;
        }

        Err(eyre!("No NSIS first header found"))
    }
}
