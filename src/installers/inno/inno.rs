use std::io::Cursor;
use std::mem;

use byteorder::{LittleEndian, ReadBytesExt};
use color_eyre::eyre::{bail, eyre, OptionExt};
use color_eyre::Result;
use crc32fast::Hasher;
use liblzma::read::XzDecoder;
use liblzma::stream::{Filters, LzmaOptions, Stream};
use yara_x::mods::pe::ResourceType;
use yara_x::mods::PE;

use crate::installers::inno::header::Header;
use crate::installers::inno::inno_block_filter::InnoBlockFilter;
use crate::installers::inno::loader::{SetupLoader, SETUP_LOADER_RESOURCE};
use crate::installers::inno::version::{InnoVersion, KNOWN_VERSIONS};

const VERSION_LEN: usize = 1 << 6;

const PROPERTIES_MAX: u8 = 9 * 5 * 5;

pub struct InnoFile {
    pub uninstall_name: Option<String>,
    pub app_version: Option<String>,
    pub app_publisher: Option<String>,
    pub product_code: Option<String>,
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
            .ok_or_eyre("No setup loader resource was found")?;

        let setup_loader = SetupLoader::new(setup_loader_data)?;

        let header_offset = setup_loader.header_offset as usize;
        let version_bytes = data
            .get(header_offset..header_offset + VERSION_LEN)
            .and_then(|bytes| memchr::memchr(u8::default(), bytes).map(|len| &bytes[..len]))
            .ok_or_eyre("Invalid Inno header")?;
        let version = String::from_utf8_lossy(version_bytes);

        let known_version = KNOWN_VERSIONS
            .into_iter()
            .rfind(|know_version| know_version.name == version)
            .ok_or_else(|| eyre!("Unknown Inno Setup Version: {version}"))?;

        let mut cursor = Cursor::new(data);
        cursor.set_position((header_offset + VERSION_LEN) as u64);

        let expected_checksum = cursor.read_u32::<LittleEndian>()?;

        let mut actual_checksum = Hasher::new();

        let mut compression = CompressionType::Stored;
        let mut stored_size = 0;
        if known_version > InnoVersion(4, 0, 9) {
            stored_size = cursor.read_u32::<LittleEndian>()?;
            actual_checksum.update(&stored_size.to_le_bytes());

            let compressed = cursor.read_u8()?;
            actual_checksum.update(&compressed.to_le_bytes());

            compression = if compressed != 0 {
                if known_version > InnoVersion(4, 1, 6) {
                    CompressionType::LZMA1
                } else {
                    CompressionType::Zlib
                }
            } else {
                CompressionType::Stored
            };
        }

        let actual_checksum = actual_checksum.finalize();
        if expected_checksum != actual_checksum {
            bail!(
                "CRC32 checksum mismatch. Expected: {expected_checksum}. Actual: {actual_checksum}"
            );
        }

        // The LZMA1 streams used by Inno Setup differ slightly from the LZMA Alone file format:
        // The stream header only stores the properties (lc, lp, pb) and the dictionary size and
        // is missing the uncompressed size field.
        let mut header = Header::default();
        if compression == CompressionType::LZMA1 {
            let mut block_filter = InnoBlockFilter::new(cursor);
            let properties = block_filter.read_u8()?;
            if properties >= PROPERTIES_MAX {
                bail!(
                    "LZMA properties value must be less than {PROPERTIES_MAX} but was {properties}",
                )
            }
            let pb = (properties / (9 * 5)) as u32;
            let lp = ((properties % (9 * 5)) / 9) as u32;
            let lc = (properties % 9) as u32;
            if lc + lp > 4 {
                bail!(
                    "LZMA lc + lp must not be greater than 4 but was {}",
                    lc + lp
                )
            }
            let dictionary_size = block_filter.read_u32::<LittleEndian>()?;
            let mut lzma_options = LzmaOptions::new();
            lzma_options.position_bits(pb);
            lzma_options.literal_position_bits(lp);
            lzma_options.literal_context_bits(lc);
            lzma_options.dict_size(dictionary_size);
            let mut filters = Filters::new();
            filters.lzma1(&lzma_options);
            let stream = Stream::new_raw_decoder(&filters)?;
            let mut decompressor = XzDecoder::new_stream(block_filter, stream);
            header = Header::load(&mut decompressor, &known_version)?;
        }

        Ok(InnoFile {
            uninstall_name: mem::take(&mut header.uninstall_name),
            app_version: mem::take(&mut header.app_version),
            app_publisher: mem::take(&mut header.app_publisher),
            product_code: mem::take(&mut header.app_id).map(|app_id| to_product_code(app_id)),
        })
    }
}

pub fn to_product_code(mut app_id: String) -> String {
    // Remove escaped bracket
    if app_id.starts_with("{{") {
        app_id.remove(0);
    }

    // Inno tags '_is1' onto the end of the app_id to create the Uninstall registry key
    app_id.push_str("_is1");
    app_id
}

#[derive(Debug, PartialEq)]
enum CompressionType {
    Stored,
    Zlib,
    LZMA1,
}
