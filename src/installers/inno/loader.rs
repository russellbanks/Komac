use std::io::{Cursor, Read};

use byteorder::{LittleEndian, ReadBytesExt};
use color_eyre::eyre::eyre;
use color_eyre::Result;
use crc32fast::Hasher;

pub const SETUP_LOADER_RESOURCE: u32 = 11111;

const SIGNATURE_LEN: usize = 12;

pub struct SetupLoaderVersion {
    signature: [u8; SIGNATURE_LEN],
    version: LoaderVersion,
}

#[derive(Debug, Default, PartialEq, PartialOrd)]
struct LoaderVersion(pub u8, pub u8, pub u8);

const KNOWN_SETUP_LOADER_VERSIONS: [SetupLoaderVersion; 7] = [
    SetupLoaderVersion {
        signature: [
            b'r', b'D', b'l', b'P', b't', b'S', b'0', b'2', 0x87, b'e', b'V', b'x',
        ],
        version: LoaderVersion(1, 2, 10),
    },
    SetupLoaderVersion {
        signature: [
            b'r', b'D', b'l', b'P', b't', b'S', b'0', b'4', 0x87, b'e', b'V', b'x',
        ],
        version: LoaderVersion(4, 0, 0),
    },
    SetupLoaderVersion {
        signature: [
            b'r', b'D', b'l', b'P', b't', b'S', b'0', b'5', 0x87, b'e', b'V', b'x',
        ],
        version: LoaderVersion(4, 0, 3),
    },
    SetupLoaderVersion {
        signature: [
            b'r', b'D', b'l', b'P', b't', b'S', b'0', b'6', 0x87, b'e', b'V', b'x',
        ],
        version: LoaderVersion(4, 0, 10),
    },
    SetupLoaderVersion {
        signature: [
            b'r', b'D', b'l', b'P', b't', b'S', b'0', b'7', 0x87, b'e', b'V', b'x',
        ],
        version: LoaderVersion(4, 1, 6),
    },
    SetupLoaderVersion {
        signature: [
            b'r', b'D', b'l', b'P', b't', b'S', 0xCD, 0xE6, 0xD7, b'{', 0x0B, b'*',
        ],
        version: LoaderVersion(5, 1, 5),
    },
    SetupLoaderVersion {
        signature: [
            b'n', b'S', b'5', b'W', b'7', b'd', b'T', 0x83, 0xAA, 0x1B, 0x0F, b'j',
        ],
        version: LoaderVersion(5, 1, 5),
    },
];

pub struct SetupLoader {
    pub setup_loader_version: SetupLoaderVersion,
    revision: u32,
    pub exe_offset: u32,
    pub exe_compressed_size: u32,
    pub exe_uncompressed_size: u32,
    pub message_offset: u32,
    pub header_offset: u32,
    pub data_offset: u32,
}

impl SetupLoader {
    pub fn new(setup_loader_data: &[u8]) -> Result<Self> {
        let mut setup_loader_data = Cursor::new(setup_loader_data);
        let mut signature = [0; SIGNATURE_LEN];
        setup_loader_data.read_exact(&mut signature)?;

        let loader_version = KNOWN_SETUP_LOADER_VERSIONS
            .into_iter()
            .find(|setup_loader_version| setup_loader_version.signature == signature)
            .ok_or_else(|| eyre!("Unknown Inno Setup loader signature: {signature:?}"))?;

        let mut crc = Hasher::new();
        crc.update(&signature);

        let mut revision = 0;
        if loader_version.version >= LoaderVersion(5, 1, 5) {
            revision = setup_loader_data.read_u32::<LittleEndian>()?;
            crc.update(&revision.to_le_bytes());
        }

        crc.update(&setup_loader_data.read_u32::<LittleEndian>()?.to_le_bytes());

        let exe_offset = setup_loader_data.read_u32::<LittleEndian>()?;
        crc.update(&exe_offset.to_le_bytes());

        let mut exe_compressed_size = 0;
        if loader_version.version < LoaderVersion(4, 1, 6) {
            exe_compressed_size = setup_loader_data.read_u32::<LittleEndian>()?;
            crc.update(&exe_compressed_size.to_le_bytes());
        }

        let exe_uncompressed_size = setup_loader_data.read_u32::<LittleEndian>()?;
        crc.update(&exe_uncompressed_size.to_le_bytes());

        if loader_version.version >= LoaderVersion(4, 0, 3) {
            let crc32 = setup_loader_data.read_u32::<LittleEndian>()?;
            crc.update(&crc32.to_le_bytes());
        }

        let message_offset = if loader_version.version >= LoaderVersion(4, 0, 0) {
            0
        } else {
            setup_loader_data.read_u32::<LittleEndian>()?
        };

        let header_offset = setup_loader_data.read_u32::<LittleEndian>()?;
        crc.update(&header_offset.to_le_bytes());

        let data_offset = setup_loader_data.read_u32::<LittleEndian>()?;
        crc.update(&data_offset.to_le_bytes());

        if loader_version.version >= LoaderVersion(4, 0, 10) {
            let expected = setup_loader_data.read_u32::<LittleEndian>()?;
            assert_eq!(crc.finalize(), expected);
        }

        Ok(Self {
            setup_loader_version: loader_version,
            revision,
            exe_offset,
            exe_compressed_size,
            exe_uncompressed_size,
            message_offset,
            header_offset,
            data_offset,
        })
    }
}
