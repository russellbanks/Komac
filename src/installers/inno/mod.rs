mod block_filter;
mod header;
mod loader;
mod lzma;
mod version;
mod windows_version;

use std::collections::BTreeSet;
use std::io::Cursor;
use std::mem;

use byteorder::{LittleEndian, ReadBytesExt};
use color_eyre::eyre::{bail, eyre, OptionExt};
use color_eyre::Result;
use crc32fast::Hasher;
use flate2::read::ZlibDecoder;
use liblzma::read::XzDecoder;
use yara_x::mods::pe::ResourceType;
use yara_x::mods::PE;

use crate::installers::inno::block_filter::{InnoBlockFilter, INNO_BLOCK_SIZE};
use crate::installers::inno::header::Header;
use crate::installers::inno::loader::{SetupLoader, SETUP_LOADER_RESOURCE};
use crate::installers::inno::lzma::read_inno_lzma_stream_header;
use crate::installers::inno::version::{InnoVersion, KnownVersion};
use crate::manifests::installer_manifest::{ElevationRequirement, UnsupportedOSArchitecture};
use crate::types::architecture::Architecture;

const VERSION_LEN: usize = 1 << 6;

const MAX_SUPPORTED_VERSION: InnoVersion = InnoVersion(6, 3, u8::MAX);

pub struct InnoFile {
    pub architecture: Option<Architecture>,
    pub unsupported_architectures: Option<BTreeSet<UnsupportedOSArchitecture>>,
    pub uninstall_name: Option<String>,
    pub app_version: Option<String>,
    pub app_publisher: Option<String>,
    pub product_code: Option<String>,
    pub elevation_requirement: Option<ElevationRequirement>,
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
            .ok_or_eyre("Invalid Inno header version")?;

        let known_version = KnownVersion::from_version_bytes(version_bytes).ok_or_else(|| {
            eyre!(
                "Unknown Inno Setup version: {}",
                &String::from_utf8_lossy(version_bytes)
            )
        })?;

        if known_version > MAX_SUPPORTED_VERSION {
            bail!("Inno Setup version {known_version} is newer than the maximum supported version {MAX_SUPPORTED_VERSION}");
        }

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
        } else {
            let compressed_size = cursor.read_u32::<LittleEndian>()?;
            actual_checksum.update(&stored_size.to_le_bytes());

            let uncompressed_size = cursor.read_u32::<LittleEndian>()?;
            actual_checksum.update(&stored_size.to_le_bytes());

            if compressed_size == u32::MAX {
                stored_size = uncompressed_size;
                compression = CompressionType::Stored;
            } else {
                stored_size = compressed_size;
                compression = CompressionType::Zlib;
            }

            // Add the size of a CRC32 checksum for each 4KiB sub-block
            stored_size += stored_size.div_ceil(u32::from(INNO_BLOCK_SIZE)) * 4;
        }

        let actual_checksum = actual_checksum.finalize();
        if expected_checksum != actual_checksum {
            bail!(
                "CRC32 checksum mismatch. Expected: {expected_checksum}. Actual: {actual_checksum}"
            );
        }

        let mut block_filter = InnoBlockFilter::new(cursor);
        let mut header = if compression == CompressionType::LZMA1 {
            let stream = read_inno_lzma_stream_header(&mut block_filter)?;
            let mut decompressor = XzDecoder::new_stream(block_filter, stream);
            Header::load(&mut decompressor, &known_version)?
        } else if compression == CompressionType::Zlib {
            let mut zlib_decoder = ZlibDecoder::new(block_filter);
            Header::load(&mut zlib_decoder, &known_version)?
        } else {
            Header::load(&mut block_filter, &known_version)?
        };

        Ok(Self {
            architecture: mem::take(&mut header.architectures_allowed).to_winget_architecture(),
            unsupported_architectures: mem::take(&mut header.architectures_disallowed)
                .to_unsupported_architectures(),
            uninstall_name: mem::take(&mut header.uninstall_name),
            app_version: mem::take(&mut header.app_version),
            app_publisher: mem::take(&mut header.app_publisher),
            product_code: mem::take(&mut header.app_id).map(to_product_code),
            elevation_requirement: mem::take(&mut header.privileges_required)
                .to_elevation_requirement(&header.privileges_required_overrides_allowed),
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
