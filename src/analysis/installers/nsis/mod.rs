mod entry;
mod file_system;
mod first_header;
mod header;
mod language;
mod registry;
mod section;
mod state;
mod strings;
mod variables;
mod version;

use std::{
    io,
    io::{Read, Seek, SeekFrom},
};

use byteorder::{LE, ReadBytesExt};
use bzip2::read::BzDecoder;
use camino::{Utf8Path, Utf8PathBuf};
use flate2::{Decompress, read::ZlibDecoder};
use liblzma::read::XzDecoder;
use msi::Language;
use protobuf::Enum;
use registry::Registry;
use state::NsisState;
use strsim::levenshtein;
use thiserror::Error;
use tracing::{debug, error};
use variables::Variables;
use winget_types::{
    LanguageTag,
    installer::{
        AppsAndFeaturesEntries, AppsAndFeaturesEntry, Architecture, InstallationMetadata,
        Installer, InstallerType, Scope,
    },
};
use yara_x::mods::{PE, pe::Machine};
use zerocopy::FromBytes;

use super::{
    super::extensions::EXE,
    nsis::{
        entry::{Entry, EntryError},
        file_system::Item,
        first_header::FirstHeader,
        header::{
            Compression, Decoder, Decompressed, Header, block::BlockHeaders,
            flags::CommonHeaderFlags,
        },
    },
    utils::{LzmaStreamHeader, RELATIVE_PROGRAM_FILES_64},
};
use crate::{analysis::Installers, traits::FromMachine};

#[derive(Error, Debug)]
pub enum NsisError {
    #[error("File is not a NSIS installer")]
    NotNsisFile,
    #[error(transparent)]
    InvalidEntry(#[from] EntryError),
    #[error("{0}")]
    ZeroCopy(String),
    #[error(transparent)]
    Io(#[from] io::Error),
}

const APP_32: &str = "app-32";
const APP_64: &str = "app-64";

pub struct Nsis {
    pub architecture: Architecture,
    pub registry: Registry,
    pub primary_language_id: u16,
    pub install_directory: Option<String>,
}

impl Nsis {
    pub fn new<R: Read + Seek>(mut reader: R, pe: &PE) -> Result<Self, NsisError> {
        let first_header_offset = pe.overlay.offset.ok_or(NsisError::NotNsisFile)?;

        // Seek to the first header
        reader
            .seek(SeekFrom::Start(first_header_offset))
            .map_err(|_| NsisError::NotNsisFile)?;

        // Read the first header
        let first_header =
            FirstHeader::try_read_from_io(&mut reader).map_err(|_| NsisError::NotNsisFile)?;

        let data_offset = first_header_offset + size_of::<FirstHeader>() as u64;

        debug!(first_header_offset, ?first_header, data_offset);

        let Decompressed {
            data: decompressed_data,
            is_solid,
            non_solid_start_offset,
            compression,
            decoder,
        } = Header::decompress(&mut reader, &first_header)?;

        let (_flags, rest) = CommonHeaderFlags::ref_from_prefix(&decompressed_data)
            .map_err(|error| NsisError::ZeroCopy(error.to_string()))?;

        let architecture = Architecture::from_machine(pe.machine());

        let (blocks, rest) = BlockHeaders::read_dynamic_from_prefix(rest, architecture)?;

        let (header, _) = Header::ref_from_prefix(rest)
            .map_err(|error| NsisError::ZeroCopy(error.to_string()))?;

        debug!(?header);

        let mut state = NsisState::new(pe, &decompressed_data, header, &blocks)?;

        // https://nsis.sourceforge.io/Reference/.onInit
        if header.code_on_init() != -1 {
            debug!("Simulating code execution for onInit callback");
            if let Err(invalid_entry) = state.execute_code_segment(header.code_on_init()) {
                error!(%invalid_entry)
            }
        }

        for (index, section) in blocks.sections(&decompressed_data).enumerate() {
            debug!(
                r#"Simulating code execution for section {index} "{}""#,
                state.get_string(section.name_offset())
            );
            match state.execute_code_segment(section.code_offset()) {
                Ok(Entry::Quit) => break,
                Err(invalid_entry) => error!(%invalid_entry),
                _ => {}
            }
        }

        // https://nsis.sourceforge.io/Reference/.onInstSuccess
        if header.code_on_inst_success() != -1 {
            debug!("Simulating code execution for onInstSuccess callback");
            if let Err(invalid_entry) = state.execute_code_segment(header.code_on_inst_success()) {
                error!(%invalid_entry)
            }
        }

        let mut architecture =
            Option::from(architecture).filter(|&architecture| architecture != Architecture::X86);

        for directory in state.file_system.directories().map(Item::name) {
            // If there is an app-64 file, the app is x64.
            // If there is an app-32 file or both files are present, the app is x86
            // (x86 apps can still install on x64 systems)
            if directory == APP_64 && architecture.is_none() {
                architecture = Some(Architecture::X64);
            } else if directory == APP_32 {
                architecture = Some(Architecture::X86);
            }
        }
        for file in state.file_system.files().map(Item::name) {
            if file.contains(APP_64) && architecture.is_none() {
                architecture = Some(Architecture::X64);
            } else if file.contains(APP_32) {
                architecture = Some(Architecture::X86);
            }
        }

        debug!(%state.file_system);

        architecture = architecture
            .or_else(|| {
                state
                    .variables
                    .install_dir()
                    .is_some_and(|dir| dir.contains(RELATIVE_PROGRAM_FILES_64))
                    .then_some(Architecture::X64)
            })
            .or_else(|| {
                let app_name = state.get_string(state.language_table.name_offset()?);
                state
                    .file_system
                    .files()
                    .filter(|file| {
                        Utf8Path::new(file.name())
                            .extension()
                            .is_some_and(|extension| extension.eq_ignore_ascii_case(EXE))
                    })
                    .min_by_key(|file| levenshtein(file.name(), &app_name))
                    .and_then(|file| {
                        let mut position = file.position()?;
                        if !is_solid {
                            position += data_offset
                                + u64::from(non_solid_start_offset)
                                + size_of::<u32>() as u64;
                        }

                        let mut decoder = if is_solid {
                            let decoder = decoder.into_inner();
                            decoder.seek(SeekFrom::Start(position)).ok()?;
                            match compression {
                                Compression::Lzma(filter_flag) => {
                                    decoder
                                        .seek_relative(position as i64 + i64::from(filter_flag))
                                        .ok()?;
                                    let stream = LzmaStreamHeader::from_reader(decoder).ok()?;
                                    Decoder::Lzma(XzDecoder::new_stream(decoder, stream))
                                }
                                Compression::BZip2 => Decoder::BZip2(BzDecoder::new(decoder)),
                                Compression::Zlib => {
                                    Decoder::Zlib(ZlibDecoder::new_with_decompress(
                                        decoder,
                                        Decompress::new(false),
                                    ))
                                }
                                Compression::None => Decoder::None(decoder),
                            }
                        } else {
                            decoder
                        };

                        let mut void = io::sink();

                        if is_solid {
                            // Seek to file
                            io::copy(&mut decoder.by_ref().take(position), &mut void).ok()?;
                        }

                        // Seek to COFF header offset inside exe
                        io::copy(&mut decoder.by_ref().take(0x3C), &mut void).ok()?;

                        let coff_offset = decoder.read_u32::<LE>().ok()?;

                        // Seek to machine value
                        io::copy(
                            &mut decoder
                                .by_ref()
                                .take(u64::from(coff_offset.checked_sub(0x3C)?)),
                            &mut void,
                        )
                        .ok()?;

                        let machine = decoder.read_u16::<LE>().ok()?;
                        Machine::from_i32(machine.into())
                    })
                    .map(Architecture::from_machine)
            });

        Ok(Self {
            architecture: architecture.unwrap_or(Architecture::X86),
            registry: state.registry,
            install_directory: state.variables.install_dir().map(str::to_owned),
            primary_language_id: state.language_table.id(),
        })
    }

    pub fn display_name(&self) -> Option<&registry::Value> {
        const DISPLAY_NAME: &str = "DisplayName";

        self.registry.get_value_by_name(DISPLAY_NAME)
    }
}

impl Installers for Nsis {
    fn installers(&self) -> Vec<Installer> {
        let product_code = self.registry.product_code();
        let display_name = self.display_name();
        let publisher = self.registry.get_value_by_name("Publisher");
        let display_version = self.registry.get_value_by_name("DisplayVersion");

        let installer = Installer {
            locale: Language::from_code(self.primary_language_id)
                .tag()
                .parse::<LanguageTag>()
                .ok(),
            architecture: self.architecture,
            r#type: Some(InstallerType::Nullsoft),
            scope: self
                .install_directory
                .as_deref()
                .and_then(Scope::from_install_directory),
            product_code: product_code.map(str::to_owned),
            apps_and_features_entries: if display_name.is_some()
                || publisher.is_some()
                || display_version.is_some()
            {
                AppsAndFeaturesEntry::builder()
                    .maybe_display_name(display_name.cloned())
                    .maybe_publisher(publisher.cloned())
                    .maybe_display_version(display_version.cloned())
                    .maybe_product_code(product_code)
                    .build()
                    .into()
            } else {
                AppsAndFeaturesEntries::new()
            },
            installation_metadata: InstallationMetadata {
                default_install_location: self.install_directory.as_deref().map(Utf8PathBuf::from),
                ..InstallationMetadata::default()
            },
            ..Installer::default()
        };

        vec![installer]
    }
}
