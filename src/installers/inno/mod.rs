mod compression;
mod encoding;
mod entry;
mod enum_value;
mod flag_reader;
mod header;
mod loader;
pub mod read;
mod version;
mod windows_version;
mod wizard;

use std::{io, io::Cursor, mem};

use camino::Utf8PathBuf;
use compact_str::CompactString;
use const_format::formatcp;
use encoding_rs::{UTF_16LE, WINDOWS_1252};
use entry::language::Language;
use itertools::Itertools;
use msi::Language as CodePageLanguage;
use thiserror::Error;
use tracing::{debug, trace};
use winget_types::{
    LanguageTag, Sha256String, Version,
    installer::{
        AppsAndFeaturesEntry, InstallationMetadata, Installer, InstallerType, Scope,
        switches::{CustomSwitch, InstallerSwitches},
    },
    url::DecodedUrl,
};
use yara_x::mods::{PE, pe::ResourceType};
use zerocopy::TryFromBytes;

use crate::installers::{
    inno::{
        entry::{
            component::Component, directory::Directory, file::File, icon::Icon, ini::Ini,
            message::Message, permission::Permission, registry::Registry, task::Task, r#type::Type,
        },
        header::{Header, flags::PrivilegesRequiredOverrides},
        loader::{SETUP_LOADER_OFFSET, SETUP_LOADER_RESOURCE, SetupLoader, SetupLoaderOffset},
        read::block::InnoBlockReader,
        version::InnoVersion,
        wizard::Wizard,
    },
    utils::{
        RELATIVE_APP_DATA, RELATIVE_COMMON_FILES_32, RELATIVE_COMMON_FILES_64,
        RELATIVE_LOCAL_APP_DATA, RELATIVE_PROGRAM_DATA, RELATIVE_PROGRAM_FILES_32,
        RELATIVE_PROGRAM_FILES_64, RELATIVE_SYSTEM_DRIVE, RELATIVE_SYSTEM_ROOT,
        RELATIVE_WINDOWS_DIR,
    },
};

const VERSION_LEN: usize = 1 << 6;

const MAX_SUPPORTED_VERSION: InnoVersion = InnoVersion::new(6, 4, u8::MAX, 0);

#[derive(Error, Debug)]
pub enum InnoError {
    #[error("File is not an Inno installer")]
    NotInnoFile,
    #[error("Invalid Inno header version")]
    InvalidSetupHeader,
    #[error(
        "Inno Setup version {0} is newer than the maximum supported version {MAX_SUPPORTED_VERSION}"
    )]
    UnsupportedVersion(InnoVersion),
    #[error("Unknown Inno setup version: {0}")]
    UnknownVersion(String),
    #[error("Unknown Inno Setup loader signature: {0:?}")]
    UnknownLoaderSignature([u8; 12]),
    #[error("Inno CRC32 checksum mismatch. Expected {expected} but calculated {actual}")]
    CrcChecksumMismatch { actual: u32, expected: u32 },
    #[error(transparent)]
    Io(#[from] io::Error),
}

pub struct Inno {
    pub installers: Vec<Installer>,
}

impl Inno {
    pub fn new(data: &[u8], pe: &PE) -> Result<Self, InnoError> {
        // Before Inno 5.1.5, the offset table is found by following a pointer at a constant offset
        let setup_loader = data
            .get(SETUP_LOADER_OFFSET..SETUP_LOADER_OFFSET + size_of::<SetupLoaderOffset>())
            .and_then(|data| SetupLoaderOffset::try_ref_from_bytes(data).ok())
            .filter(|offset| offset.table_offset == !offset.not_table_offset)
            .and_then(|offset| data.get(offset.table_offset.get() as usize..))
            .map(SetupLoader::new)
            .or_else(|| {
                // From Inno 5.1.5, the offset table is stored as a PE resource entry
                pe.resources
                    .iter()
                    .filter(|resource| resource.type_() == ResourceType::RESOURCE_TYPE_RCDATA)
                    .find(|resource| resource.id() == SETUP_LOADER_RESOURCE)
                    .and_then(|resource| {
                        let offset = resource.offset() as usize;
                        data.get(offset..offset + resource.length() as usize)
                    })
                    .map(SetupLoader::new)
            })
            .ok_or(InnoError::NotInnoFile)??;

        let header_offset = setup_loader.header_offset as usize;
        let version_bytes = data
            .get(header_offset..header_offset + VERSION_LEN)
            .and_then(|bytes| memchr::memchr(0, bytes).map(|len| &bytes[..len]))
            .ok_or(InnoError::InvalidSetupHeader)?;

        debug!(raw_version = ?std::str::from_utf8(version_bytes));

        let inno_version = InnoVersion::from_version_bytes(version_bytes).ok_or_else(|| {
            InnoError::UnknownVersion(String::from_utf8_lossy(version_bytes).into_owned())
        })?;

        debug!(?inno_version);

        if inno_version > MAX_SUPPORTED_VERSION {
            return Err(InnoError::UnsupportedVersion(inno_version));
        }

        let mut cursor = Cursor::new(data);
        cursor.set_position((header_offset + VERSION_LEN) as u64);

        let mut reader = InnoBlockReader::get(cursor, &inno_version)?;

        let mut codepage = if inno_version.is_unicode() {
            UTF_16LE
        } else {
            WINDOWS_1252
        };

        let mut header = Header::from_reader(&mut reader, codepage, &inno_version)?;

        debug!(?header);

        let languages = (0..header.language_count)
            .map(|_| Language::from_reader(&mut reader, codepage, &inno_version))
            .collect::<io::Result<Vec<_>>>()?;

        if !inno_version.is_unicode() {
            codepage = languages
                .iter()
                .map(|language| language.codepage)
                .find_or_first(|&codepage| codepage == WINDOWS_1252)
                .unwrap_or(WINDOWS_1252);
        }

        if inno_version < (4, 0, 0) {
            debug!("Reading images and plugins");
            Wizard::from_reader(&mut reader, &inno_version, &header)?;
        }

        trace!("Reading messages");
        let _messages = (0..header.message_count)
            .map(|_| Message::from_reader(&mut reader, &languages, codepage))
            .collect::<io::Result<Vec<_>>>()?;

        trace!("Reading permissions");
        let _permissions = (0..header.permission_count)
            .map(|_| Permission::from_reader(&mut reader, codepage))
            .collect::<io::Result<Vec<_>>>()?;

        trace!("Reading type entries");
        let _type_entries = (0..header.type_count)
            .map(|_| Type::from_reader(&mut reader, codepage, &inno_version))
            .collect::<io::Result<Vec<_>>>()?;

        trace!("Reading components");
        let _components = (0..header.component_count)
            .map(|_| Component::from_reader(&mut reader, codepage, &inno_version))
            .collect::<io::Result<Vec<_>>>()?;

        trace!("Reading tasks");
        let _tasks = (0..header.task_count)
            .map(|_| Task::from_reader(&mut reader, codepage, &inno_version))
            .collect::<io::Result<Vec<_>>>()?;

        trace!("Reading directories");
        let _directories = (0..header.directory_count)
            .map(|_| Directory::from_reader(&mut reader, codepage, &inno_version))
            .collect::<io::Result<Vec<_>>>()?;

        trace!("Reading files");
        let _files = (0..header.file_count)
            .map(|_| File::from_reader(&mut reader, codepage, &inno_version))
            .collect::<io::Result<Vec<_>>>()?;

        trace!("Reading icons");
        let _icons = (0..header.icon_count)
            .map(|_| Icon::from_reader(&mut reader, codepage, &inno_version))
            .collect::<io::Result<Vec<_>>>()?;

        trace!("Reading ini entries");
        let _ini = (0..header.ini_entry_count)
            .map(|_| Ini::from_reader(&mut reader, codepage, &inno_version))
            .collect::<io::Result<Vec<_>>>()?;

        trace!("Reading registry entries");
        let _registry = (0..header.registry_entry_count)
            .map(|_| Registry::from_reader(&mut reader, codepage, &inno_version))
            .collect::<io::Result<Vec<_>>>()?;

        let install_dir = header
            .default_dir_name
            .take()
            .map(to_relative_install_dir)
            .filter(|dir| !dir.contains(['{', '}']));

        let mut installer = Installer {
            locale: languages.first().and_then(|language_entry| {
                CodePageLanguage::from_code(u16::try_from(language_entry.id).ok()?)
                    .tag()
                    .parse::<LanguageTag>()
                    .ok()
            }),
            architecture: mem::take(&mut header.architectures_allowed).into(),
            r#type: Some(InstallerType::Inno),
            scope: install_dir
                .as_deref()
                .and_then(Scope::from_install_directory),
            url: DecodedUrl::default(),
            sha_256: Sha256String::default(),
            product_code: header.app_id.clone().map(to_product_code),
            unsupported_os_architectures: header.architectures_disallowed.into(),
            apps_and_features_entries: if header.uninstall_name.is_some()
                || header.app_publisher.is_some()
                || header.app_version.is_some()
            {
                vec![AppsAndFeaturesEntry {
                    display_name: header.uninstall_name.take().map(CompactString::from),
                    publisher: header.app_publisher.take().map(CompactString::from),
                    display_version: header.app_version.as_deref().map(Version::new),
                    product_code: header.app_id.take().map(to_product_code),
                    ..AppsAndFeaturesEntry::default()
                }]
            } else {
                vec![]
            },
            elevation_requirement: header
                .privileges_required
                .to_elevation_requirement(&header.privileges_required_overrides_allowed),
            installation_metadata: InstallationMetadata {
                default_install_location: install_dir.map(Utf8PathBuf::from),
                ..InstallationMetadata::default()
            },
            ..Default::default()
        };

        let installers = if header.privileges_required_overrides_allowed.is_empty() {
            vec![installer]
        } else {
            installer.scope = Some(Scope::Machine);
            let has_scope_switch = header
                .privileges_required_overrides_allowed
                .contains(PrivilegesRequiredOverrides::COMMAND_LINE);
            if has_scope_switch {
                installer.switches = InstallerSwitches {
                    custom: Some(CustomSwitch::all_users()),
                    ..InstallerSwitches::default()
                };
            }
            let user_installer = Installer {
                scope: Some(Scope::User),
                switches: InstallerSwitches {
                    custom: has_scope_switch.then(CustomSwitch::current_user),
                    ..InstallerSwitches::default()
                },
                installation_metadata: InstallationMetadata::default(),
                ..installer.clone()
            };
            vec![installer, user_installer]
        };

        Ok(Self { installers })
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
    const SYSTEM_DRIVE: &str = "{sd}";

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
        (SYSTEM_DRIVE, RELATIVE_SYSTEM_DRIVE),
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
        (LOCAL_APP_DATA, RELATIVE_LOCAL_APP_DATA),
        (USER_APP_DATA, RELATIVE_APP_DATA),
        (COMMON_APP_DATA, RELATIVE_PROGRAM_DATA),
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
