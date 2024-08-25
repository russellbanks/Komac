use std::cmp::Ordering;
use std::io::{Cursor, Read};

use crate::installers::inno::read::crc32::Crc32Reader;
use crate::installers::inno::version::InnoVersion;
use byteorder::{LittleEndian, ReadBytesExt};
use color_eyre::eyre::{bail, eyre};
use color_eyre::Result;

pub const SETUP_LOADER_RESOURCE: u32 = 11111;

const SIGNATURE_LEN: usize = 12;

pub struct SetupLoaderVersion {
    signature: [u8; SIGNATURE_LEN],
    version: InnoVersion,
}

impl PartialEq<InnoVersion> for SetupLoaderVersion {
    fn eq(&self, other: &InnoVersion) -> bool {
        self.version.eq(other)
    }
}

impl PartialOrd<InnoVersion> for SetupLoaderVersion {
    fn partial_cmp(&self, other: &InnoVersion) -> Option<Ordering> {
        self.version.partial_cmp(other)
    }
}

const KNOWN_SETUP_LOADER_VERSIONS: [SetupLoaderVersion; 7] = [
    SetupLoaderVersion {
        signature: [
            b'r', b'D', b'l', b'P', b't', b'S', b'0', b'2', 0x87, b'e', b'V', b'x',
        ],
        version: InnoVersion(1, 2, 10),
    },
    SetupLoaderVersion {
        signature: [
            b'r', b'D', b'l', b'P', b't', b'S', b'0', b'4', 0x87, b'e', b'V', b'x',
        ],
        version: InnoVersion(4, 0, 0),
    },
    SetupLoaderVersion {
        signature: [
            b'r', b'D', b'l', b'P', b't', b'S', b'0', b'5', 0x87, b'e', b'V', b'x',
        ],
        version: InnoVersion(4, 0, 3),
    },
    SetupLoaderVersion {
        signature: [
            b'r', b'D', b'l', b'P', b't', b'S', b'0', b'6', 0x87, b'e', b'V', b'x',
        ],
        version: InnoVersion(4, 0, 10),
    },
    SetupLoaderVersion {
        signature: [
            b'r', b'D', b'l', b'P', b't', b'S', b'0', b'7', 0x87, b'e', b'V', b'x',
        ],
        version: InnoVersion(4, 1, 6),
    },
    SetupLoaderVersion {
        signature: [
            b'r', b'D', b'l', b'P', b't', b'S', 0xCD, 0xE6, 0xD7, b'{', 0x0B, b'*',
        ],
        version: InnoVersion(5, 1, 5),
    },
    SetupLoaderVersion {
        signature: [
            b'n', b'S', b'5', b'W', b'7', b'd', b'T', 0x83, 0xAA, 0x1B, 0x0F, b'j',
        ],
        version: InnoVersion(5, 1, 5),
    },
];

#[allow(unused)]
enum Checksum {
    Adler32(u32),
    CRC32(u32),
}

pub struct SetupLoader {
    setup_loader_version: SetupLoaderVersion,
    revision: u32,
    exe_offset: u32,
    exe_compressed_size: u32,
    exe_uncompressed_size: u32,
    exe_checksum: Checksum,
    message_offset: u32,
    pub header_offset: u32,
    data_offset: u32,
}

impl SetupLoader {
    pub fn new(setup_loader_data: &[u8]) -> Result<Self> {
        let mut checksum = Crc32Reader::new(Cursor::new(setup_loader_data));
        let mut signature = [0; SIGNATURE_LEN];
        checksum.read_exact(&mut signature)?;

        let loader_version = KNOWN_SETUP_LOADER_VERSIONS
            .into_iter()
            .find(|setup_loader_version| setup_loader_version.signature == signature)
            .ok_or_else(|| eyre!("Unknown Inno Setup loader signature: {signature:?}"))?;

        let revision = if loader_version >= InnoVersion(5, 1, 5) {
            checksum.read_u32::<LittleEndian>()?
        } else {
            0
        };

        checksum.read_u32::<LittleEndian>()?;
        let exe_offset = checksum.read_u32::<LittleEndian>()?;

        let exe_compressed_size = if loader_version >= InnoVersion(4, 1, 6) {
            0
        } else {
            checksum.read_u32::<LittleEndian>()?
        };

        let exe_uncompressed_size = checksum.read_u32::<LittleEndian>()?;

        let exe_checksum = if loader_version >= InnoVersion(4, 0, 3) {
            Checksum::CRC32(checksum.read_u32::<LittleEndian>()?)
        } else {
            Checksum::Adler32(checksum.read_u32::<LittleEndian>()?)
        };

        let message_offset = if loader_version >= InnoVersion(4, 0, 0) {
            0
        } else {
            checksum.get_mut().read_u32::<LittleEndian>()?
        };

        let header_offset = checksum.read_u32::<LittleEndian>()?;
        let data_offset = checksum.read_u32::<LittleEndian>()?;

        if loader_version >= InnoVersion(4, 0, 10) {
            let expected_checksum = checksum.get_mut().read_u32::<LittleEndian>()?;
            let actual_checksum = checksum.finalize();
            if actual_checksum != expected_checksum {
                bail!("CRC32 checksum mismatch. Actual: {actual_checksum}. Expected: {expected_checksum}.")
            }
        }

        Ok(Self {
            setup_loader_version: loader_version,
            revision,
            exe_offset,
            exe_compressed_size,
            exe_uncompressed_size,
            exe_checksum,
            message_offset,
            header_offset,
            data_offset,
        })
    }
}
