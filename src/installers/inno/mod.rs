mod compression;
mod encoding;
mod header;
mod language;
mod loader;
pub mod read;
mod version;
mod windows_version;

use crate::installers::inno::compression::Compression;
use crate::installers::inno::header::Header;
use crate::installers::inno::language::LanguageEntry;
use crate::installers::inno::loader::{SetupLoader, SETUP_LOADER_RESOURCE};
use crate::installers::inno::read::block_filter::{InnoBlockFilter, INNO_BLOCK_SIZE};
use crate::installers::inno::read::crc32::Crc32Reader;
use crate::installers::inno::version::{InnoVersion, KnownVersion};
use crate::installers::inno::InnoError::{UnknownVersion, UnsupportedVersion};
use crate::installers::traits::InstallSpec;
use crate::installers::utils::{
    read_lzma_stream_header, RELATIVE_APP_DATA, RELATIVE_COMMON_FILES_32, RELATIVE_COMMON_FILES_64,
    RELATIVE_LOCAL_APP_DATA, RELATIVE_PROGRAM_FILES_32, RELATIVE_PROGRAM_FILES_64,
    RELATIVE_SYSTEM_ROOT, RELATIVE_TEMP_FOLDER, RELATIVE_WINDOWS_DIR,
};
use crate::manifests::installer_manifest::{
    ElevationRequirement, Scope, UnsupportedOSArchitecture,
};
use crate::types::architecture::Architecture;
use crate::types::installer_type::InstallerType;
use crate::types::language_tag::LanguageTag;
use byteorder::{ReadBytesExt, LE};
use camino::Utf8PathBuf;
use const_format::formatcp;
use flate2::read::ZlibDecoder;
use liblzma::read::XzDecoder;
use msi::Language;
use std::collections::BTreeSet;
use std::io::{Cursor, Read};
use std::str::FromStr;
use std::{io, mem};
use thiserror::Error;
use yara_x::mods::pe::ResourceType;
use yara_x::mods::PE;

const VERSION_LEN: usize = 1 << 6;

const MAX_SUPPORTED_VERSION: InnoVersion = InnoVersion(6, 3, u8::MAX);

#[derive(Error, Debug)]
pub enum InnoError {
    #[error("File is not an Inno installer")]
    NotInnoFile,
    #[error("Invalid Inno header version")]
    InvalidSetupHeader,
    #[error("Inno Setup version {0} is newer than the maximum supported version {MAX_SUPPORTED_VERSION}")]
    UnsupportedVersion(KnownVersion),
    #[error("Unknown Inno setup version: {0}")]
    UnknownVersion(String),
    #[error("Unknown Inno Setup loader signature: {0:?}")]
    UnknownLoaderSignature([u8; 12]),
    #[error("CRC32 checksum mismatch. Actual: {actual}. Expected: {expected}.")]
    CrcChecksumMismatch { actual: u32, expected: u32 },
    #[error(transparent)]
    Io(#[from] io::Error),
}

pub struct Inno {
    architecture: Option<Architecture>,
    scope: Option<Scope>,
    install_dir: Option<Utf8PathBuf>,
    unsupported_architectures: Option<BTreeSet<UnsupportedOSArchitecture>>,
    uninstall_name: Option<String>,
    app_version: Option<String>,
    app_publisher: Option<String>,
    product_code: Option<String>,
    elevation_requirement: Option<ElevationRequirement>,
    installer_locale: Option<LanguageTag>,
}

impl Inno {
    pub fn new(data: &[u8], pe: &PE) -> Result<Self, InnoError> {
        let setup_loader_data = pe
            .resources
            .iter()
            .filter(|resource| resource.type_() == ResourceType::RESOURCE_TYPE_RCDATA)
            .find(|resource| resource.id() == SETUP_LOADER_RESOURCE)
            .and_then(|resource| {
                let offset = resource.offset() as usize;
                data.get(offset..offset + resource.length() as usize)
            })
            .ok_or(InnoError::NotInnoFile)?;

        let setup_loader = SetupLoader::new(setup_loader_data)?;

        let header_offset = setup_loader.header_offset as usize;
        let version_bytes = data
            .get(header_offset..header_offset + VERSION_LEN)
            .and_then(|bytes| memchr::memchr(0, bytes).map(|len| &bytes[..len]))
            .ok_or(InnoError::InvalidSetupHeader)?;

        let known_version = KnownVersion::from_version_bytes(version_bytes)
            .ok_or_else(|| UnknownVersion(String::from_utf8_lossy(version_bytes).into_owned()))?;

        if known_version > MAX_SUPPORTED_VERSION {
            return Err(UnsupportedVersion(known_version));
        }

        let mut cursor = Cursor::new(data);
        cursor.set_position((header_offset + VERSION_LEN) as u64);

        let expected_checksum = cursor.read_u32::<LE>()?;

        let mut actual_checksum = Crc32Reader::new(&mut cursor);

        let stored_size = if known_version > InnoVersion(4, 0, 9) {
            let size = actual_checksum.read_u32::<LE>()?;
            let compressed = actual_checksum.read_u8()? != 0;

            if compressed {
                if known_version > InnoVersion(4, 1, 6) {
                    Compression::LZMA1(size)
                } else {
                    Compression::Zlib(size)
                }
            } else {
                Compression::Stored(size)
            }
        } else {
            let compressed_size = actual_checksum.read_u32::<LE>()?;
            let uncompressed_size = actual_checksum.read_u32::<LE>()?;

            let mut stored_size = if compressed_size == u32::MAX {
                Compression::Stored(uncompressed_size)
            } else {
                Compression::Zlib(compressed_size)
            };

            // Add the size of a CRC32 checksum for each 4KiB sub-block
            *stored_size += stored_size.div_ceil(u32::from(INNO_BLOCK_SIZE)) * 4;

            stored_size
        };

        let actual_checksum = actual_checksum.finalize();
        if actual_checksum != expected_checksum {
            return Err(InnoError::CrcChecksumMismatch {
                actual: actual_checksum,
                expected: expected_checksum,
            });
        }

        let mut block_filter = InnoBlockFilter::new(cursor.take(u64::from(*stored_size)));
        let mut reader: Box<dyn Read> = match stored_size {
            Compression::LZMA1(_) => {
                let stream = read_lzma_stream_header(&mut block_filter)?;
                Box::new(XzDecoder::new_stream(block_filter, stream))
            }
            Compression::Zlib(_) => Box::new(ZlibDecoder::new(block_filter)),
            Compression::Stored(_) => Box::new(block_filter),
        };

        let mut header = Header::load(&mut reader, &known_version)?;

        let mut language_entries = Vec::new();
        for _ in 0..header.language_count {
            language_entries.push(LanguageEntry::load(&mut reader, &known_version)?);
        }

        let auto_install_directory = header
            .default_dir_name
            .as_deref()
            .is_some_and(|dir| dir.starts_with("{auto"));

        let install_dir = header.default_dir_name.take().map(to_relative_install_dir);

        Ok(Self {
            architecture: mem::take(&mut header.architectures_allowed).to_winget_architecture(),
            scope: if auto_install_directory {
                None
            } else {
                install_dir.as_deref().and_then(Scope::from_install_dir)
            },
            install_dir: install_dir.map(Utf8PathBuf::from),
            unsupported_architectures: mem::take(&mut header.architectures_disallowed)
                .to_unsupported_architectures(),
            uninstall_name: header.uninstall_name.take(),
            app_version: header.app_version.take(),
            app_publisher: header.app_publisher.take(),
            product_code: header.app_id.take().map(to_product_code),
            elevation_requirement: header
                .privileges_required
                .to_elevation_requirement(&header.privileges_required_overrides_allowed),
            installer_locale: language_entries.first().and_then(|language_entry| {
                LanguageTag::from_str(
                    Language::from_code(u16::try_from(language_entry.language_id).ok()?).tag(),
                )
                .ok()
            }),
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

pub fn to_relative_install_dir(mut install_dir: String) -> String {
    const WINDOWS: &str = "{win}";
    const SYSTEM: &str = "{sys}";
    const SYSTEM_NATIVE: &str = "{sysnative}";

    const PROGRAM_FILES: &str = "{commonpf}";
    const PROGRAM_FILES_32: &str = "{commonpf32}";
    const PROGRAM_FILES_64: &str = "{commonpf64}";
    const COMMON_FILES: &str = "{commoncf}";
    const COMMON_FILES_32: &str = "{commoncf32}";
    const COMMON_FILES_64: &str = "{commoncf64}";

    const PROGRAM_FILES_OLD: &str = "{pf}";
    const PROGRAM_FILES_32_OLD: &str = "{pf32}";
    const PROGRAM_FILES_64_OLD: &str = "{pf64}";
    const COMMON_FILES_OLD: &str = "{cf}";
    const COMMON_FILES_32_OLD: &str = "{cf32}";
    const COMMON_FILES_64_OLD: &str = "{cf64}";

    const AUTO_PROGRAM_FILES: &str = "{autopf}";
    const AUTO_PROGRAM_FILES_32: &str = "{autopf32}";
    const AUTO_PROGRAM_FILES_64: &str = "{autopf64}";
    const AUTO_COMMON_FILES: &str = "{autocf}";
    const AUTO_COMMON_FILES_32: &str = "{autocf32}";
    const AUTO_COMMON_FILES_64: &str = "{autocf64}";
    const AUTO_APP_DATA: &str = "{autoappdata}";

    const TEMP: &str = "{tmp}";

    const LOCAL_APP_DATA: &str = "{localappdata}";
    const USER_APP_DATA: &str = "{userappdata}";
    const COMMON_APP_DATA: &str = "{commonappdata}";

    const USER_PROGRAM_FILES: &str = "{userpf}";
    const USER_COMMON_FILES: &str = "{usercf}";

    const RELATIVE_USER_PROGRAM_FILES: &str = formatcp!(r"{RELATIVE_LOCAL_APP_DATA}\Programs");

    const DIRECTORIES: [(&str, &str); 28] = [
        (WINDOWS, RELATIVE_WINDOWS_DIR),
        (SYSTEM, RELATIVE_SYSTEM_ROOT),
        (SYSTEM_NATIVE, RELATIVE_SYSTEM_ROOT),
        (PROGRAM_FILES, RELATIVE_PROGRAM_FILES_64),
        (PROGRAM_FILES_32, RELATIVE_PROGRAM_FILES_32),
        (PROGRAM_FILES_64, RELATIVE_PROGRAM_FILES_64),
        (COMMON_FILES, RELATIVE_COMMON_FILES_64),
        (COMMON_FILES_32, RELATIVE_COMMON_FILES_32),
        (COMMON_FILES_64, RELATIVE_COMMON_FILES_64),
        (PROGRAM_FILES_OLD, RELATIVE_PROGRAM_FILES_64),
        (PROGRAM_FILES_32_OLD, RELATIVE_PROGRAM_FILES_32),
        (PROGRAM_FILES_64_OLD, RELATIVE_PROGRAM_FILES_64),
        (COMMON_FILES_OLD, RELATIVE_COMMON_FILES_64),
        (COMMON_FILES_32_OLD, RELATIVE_COMMON_FILES_32),
        (COMMON_FILES_64_OLD, RELATIVE_COMMON_FILES_64),
        (AUTO_PROGRAM_FILES, RELATIVE_PROGRAM_FILES_64),
        (AUTO_PROGRAM_FILES_32, RELATIVE_PROGRAM_FILES_32),
        (AUTO_PROGRAM_FILES_64, RELATIVE_PROGRAM_FILES_64),
        (AUTO_COMMON_FILES, RELATIVE_COMMON_FILES_64),
        (AUTO_COMMON_FILES_32, RELATIVE_COMMON_FILES_32),
        (AUTO_COMMON_FILES_64, RELATIVE_COMMON_FILES_64),
        (AUTO_APP_DATA, RELATIVE_APP_DATA),
        (TEMP, RELATIVE_TEMP_FOLDER),
        (LOCAL_APP_DATA, RELATIVE_LOCAL_APP_DATA),
        (USER_APP_DATA, RELATIVE_APP_DATA),
        (COMMON_APP_DATA, RELATIVE_APP_DATA),
        (USER_PROGRAM_FILES, RELATIVE_USER_PROGRAM_FILES),
        (
            USER_COMMON_FILES,
            formatcp!(r"{RELATIVE_USER_PROGRAM_FILES}\Common"),
        ),
    ];

    for (inno_directory, relative_directory) in DIRECTORIES {
        if let Some(index) = install_dir.find(inno_directory) {
            install_dir.replace_range(index..index + inno_directory.len(), relative_directory);
            break;
        }
    }

    install_dir
}

impl InstallSpec for Inno {
    fn r#type(&self) -> InstallerType {
        InstallerType::Inno
    }

    fn architecture(&mut self) -> Option<Architecture> {
        self.architecture.take()
    }

    fn display_name(&mut self) -> Option<String> {
        self.uninstall_name.take()
    }

    fn display_publisher(&mut self) -> Option<String> {
        self.app_publisher.take()
    }

    fn display_version(&mut self) -> Option<String> {
        self.app_version.take()
    }

    fn product_code(&mut self) -> Option<String> {
        self.product_code.take()
    }

    fn locale(&mut self) -> Option<LanguageTag> {
        self.installer_locale.take()
    }

    fn scope(&self) -> Option<Scope> {
        self.scope
    }

    fn unsupported_os_architectures(&mut self) -> Option<BTreeSet<UnsupportedOSArchitecture>> {
        self.unsupported_architectures.take()
    }

    fn elevation_requirement(&mut self) -> Option<ElevationRequirement> {
        self.elevation_requirement.take()
    }

    fn install_location(&mut self) -> Option<Utf8PathBuf> {
        self.install_dir.take()
    }
}
