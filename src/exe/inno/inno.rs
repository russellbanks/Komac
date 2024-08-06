use std::io::{Cursor, Read};

use byteorder::{LittleEndian, ReadBytesExt};
use color_eyre::eyre::{eyre, OptionExt};
use color_eyre::Result;
use crc32fast::Hasher;
use liblzma::read::XzDecoder;
use liblzma::stream::{Filters, LzmaOptions, Stream};
use yara_x::mods::PE;
use yara_x::mods::pe::ResourceType;

use crate::exe::inno::header::Header;
use crate::exe::inno::loader::{SETUP_LOADER_RESOURCE, SetupLoader};
use crate::exe::inno::version::{InnoVersion, KNOWN_VERSIONS, KnownVersion};

const VERSION_LEN: usize = 1 << 6;

pub struct InnoFile {
    version: KnownVersion,
    setup_loader: SetupLoader,
}

impl InnoFile {
    pub fn new(data: &[u8], pe: &PE) -> Result<Self> {
        let setup_loader_data = pe
            .resources
            .iter()
            .filter(|resource| resource.type_() == ResourceType::RESOURCE_TYPE_RCDATA)
            .find(|resource| resource.id() == SETUP_LOADER_RESOURCE)
            .and_then(|resource| {
                let offset = resource.offset() as usize;
                data.get(offset..offset + resource.length() as usize)
            })
            .ok_or_eyre("No Setup loader resource was found")?;

        let setup_loader = SetupLoader::new(setup_loader_data)?;

        let header_offset = setup_loader.header_offset as usize;
        let version_bytes = data
            .get(header_offset..header_offset + VERSION_LEN)
            .and_then(|bytes| memchr::memchr(u8::default(), bytes).map(|len| &bytes[..len]))
            .unwrap();
        let version = String::from_utf8_lossy(version_bytes);
        dbg!(&version);

        let known_version = KNOWN_VERSIONS
            .into_iter()
            .rfind(|know_version| know_version.name == version)
            .ok_or_else(|| eyre!("Unknown Inno Setup Version: {version}"))?;

        let mut cursor = Cursor::new(data);
        cursor.set_position((header_offset + VERSION_LEN) as u64);

        let expected_checksum = cursor.read_u32::<LittleEndian>()?;
        dbg!(expected_checksum);

        let mut actual_checksum = Hasher::new();

        let mut compression = CompressionType::Stored;
        let mut stored_size = 0;
        if known_version.version > InnoVersion(4, 0, 9, 0) {
            stored_size = cursor.read_u32::<LittleEndian>()?;
            actual_checksum.update(&stored_size.to_le_bytes());
            dbg!(&stored_size);

            let compressed = cursor.read_u8()?;
            actual_checksum.update(&compressed.to_le_bytes());
            dbg!(&compressed);

            compression = if compressed != 0 {
                if known_version.version > InnoVersion(4, 1, 6, 0) {
                    CompressionType::LZMA1
                } else {
                    CompressionType::Zlib
                }
            } else {
                CompressionType::Stored
            };
            dbg!(&compression);
        }

        assert_eq!(expected_checksum, actual_checksum.finalize());

        // The LZMA1 streams used by Inno Setup differ slightly from the LZMA Alone file format:
        // The stream header only stores the properties (lc, lp, pb) and the dictionary size and
        // is missing the uncompressed size field.
        if compression == CompressionType::LZMA1 {
            let block_expected_checksum = cursor.read_u32::<LittleEndian>()?;
            let properties = cursor.read_u8()?;
            dbg!(&properties);
            assert!(properties < (9 * 5 * 5));
            let pb = (properties / (9 * 5)) as u32;
            let lp = ((properties % (9 * 5)) / 9) as u32;
            let lc = (properties % 9) as u32;
            dbg!(&pb, &lp, &lc);
            assert!(lc + lp <= 4);
            let dictionary_size = cursor.read_u32::<LittleEndian>()?;
            dbg!(&dictionary_size);
            let mut lzma_options = LzmaOptions::new();
            lzma_options.position_bits(pb);
            lzma_options.literal_position_bits(lp);
            lzma_options.literal_context_bits(lc);
            let mut filters = Filters::new();
            filters.lzma1(&lzma_options);
            let stream = Stream::new_raw_decoder(&filters)?;
            let mut decompressor = XzDecoder::new_stream(cursor, stream);
            let header = Header::load(&mut decompressor, &known_version.version)?;
            dbg!(&header);
        }

        Ok(InnoFile {
            version: known_version,
            setup_loader,
        })
    }
}

#[derive(Debug, PartialEq)]
enum CompressionType {
    Stored,
    Zlib,
    LZMA1,
}
