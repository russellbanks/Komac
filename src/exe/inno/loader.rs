use color_eyre::Result;
use crc32fast::Hasher;
use object::ReadRef;
use versions::SemVer;

pub const SETUP_LOADER_RESOURCE: u16 = 11111;

const SIGNATURE_LEN: usize = 12;

#[derive(Default)]
struct SetupLoaderVersion {
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

pub struct SetupLoader<'data> {
    pub setup_loader_version: SetupLoaderVersion,
    revision: &'data u32,
    pub exe_offset: &'data u32,
    pub exe_compressed_size: &'data u32,
    pub exe_uncompressed_size: &'data u32,
    pub message_offset: &'data u32,
    pub header_offset: &'data u32,
    pub data_offset: &'data u32,
}

impl<'data> SetupLoader<'data> {
    pub fn new<R: ReadRef<'data>>(data: R, offset: &mut u64) -> Result<Self> {
        let signature = data.read_slice(offset, SIGNATURE_LEN).unwrap();

        let loader_version = KNOWN_SETUP_LOADER_VERSIONS
            .into_iter()
            .find(|setup_loader_version| setup_loader_version.signature == signature)
            .unwrap_or_default();

        println!("Loader version: {:?}", loader_version.version);

        let mut crc = Hasher::new();
        crc.update(signature);

        let mut revision = &0;
        if loader_version.version >= LoaderVersion(5, 1, 5) {
            revision = data.read::<u32>(offset).unwrap();
            crc.update(&revision.to_le_bytes());
            println!("Revision: {revision}");
        }

        let space = data.read::<u32>(offset).unwrap();
        crc.update(&space.to_le_bytes());

        let exe_offset = data.read::<u32>(offset).unwrap();

        println!("exe offset: {}", exe_offset);

        let mut exe_compressed_size = &0;
        if loader_version.version < LoaderVersion(4, 1, 6) {
            exe_compressed_size = data.read::<u32>(offset).unwrap();
            crc.update(&exe_compressed_size.to_le_bytes())
        }

        let exe_uncompressed_size = data.read::<u32>(offset).unwrap();
        crc.update(&exe_uncompressed_size.to_le_bytes());
        println!("exe uncompressed size: {}", exe_uncompressed_size);

        if loader_version.version >= LoaderVersion(4, 0, 3) {
            let crc32 = data.read::<u32>(offset).unwrap();
            crc.update(&crc32.to_le_bytes());
            println!("crc32: {}", crc32);
        }

        let message_offset = if loader_version.version >= LoaderVersion(4, 0, 0) {
            &0
        } else {
            data.read::<u32>(offset).unwrap()
        };

        let header_offset = data.read::<u32>(offset).unwrap();
        crc.update(&header_offset.to_le_bytes());
        println!("Header offset: {}", header_offset);

        let data_offset = data.read::<u32>(offset).unwrap();
        crc.update(&data_offset.to_le_bytes());
        println!("Data offset: {}", data_offset);

        if loader_version.version >= LoaderVersion(4, 0, 10) {
            let expected = data.read::<u32>(offset).unwrap();
            let checksum = crc.finalize();
            println!("Expected: {}", expected);
            println!("Checksum: {}", checksum);
        }

        Ok(SetupLoader {
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
