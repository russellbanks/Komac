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

use std::{io, io::Read};

use byteorder::{LE, ReadBytesExt};
use bzip2::read::BzDecoder;
use camino::{Utf8Path, Utf8PathBuf};
use flate2::{Decompress, read::ZlibDecoder};
use liblzma::read::XzDecoder;
use msi::Language;
use protobuf::Enum;
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
use zerocopy::{FromBytes, TryFromBytes};

use crate::{
    file_analyser::EXE,
    installers::{
        nsis::{
            entry::EntryError,
            file_system::Item,
            first_header::FirstHeader,
            header::{
                Decompressed, Header, block::BlockHeaders, compression::Compression,
                decoder::Decoder, flags::CommonHeaderFlags,
            },
        },
        utils::{RELATIVE_PROGRAM_FILES_64, lzma_stream_header::LzmaStreamHeader},
    },
    traits::FromMachine,
};

#[derive(Error, Debug)]
pub enum NsisError {
    #[error("File is not a NSIS installer")]
    NotNsisFile,
    #[error("Failed to get NSIS first header offset")]
    FirstHeaderOffset,
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
    pub installer: Installer,
}

impl Nsis {
    pub fn new(data: &[u8], pe: &PE) -> Result<Self, NsisError> {
        let first_header_offset = pe
            .overlay
            .offset
            .and_then(|offset| usize::try_from(offset).ok())
            .ok_or(NsisError::FirstHeaderOffset)?;

        let data_offset = first_header_offset + size_of::<FirstHeader>();
        let first_header = data
            .get(first_header_offset..data_offset)
            .ok_or(NsisError::NotNsisFile)
            .and_then(|bytes| {
                FirstHeader::try_ref_from_bytes(bytes).map_err(|_| NsisError::NotNsisFile)
            })?;

        debug!(first_header_offset, ?first_header, data_offset);

        let Decompressed {
            data: decompressed_data,
            is_solid,
            non_solid_start_offset,
            compression,
            decoder: solid_decoder,
        } = Header::decompress(&data[data_offset..], first_header)?;

        let (_flags, rest) = CommonHeaderFlags::ref_from_prefix(&decompressed_data)
            .map_err(|error| NsisError::ZeroCopy(error.to_string()))?;

        let architecture = Architecture::from_machine(pe.machine());

        let (blocks, rest) = BlockHeaders::read_dynamic_from_prefix(rest, architecture)?;

        let (header, _) = Header::ref_from_prefix(rest)
            .map_err(|error| NsisError::ZeroCopy(error.to_string()))?;

        debug!(?header);

        let mut state = NsisState::new(pe, &decompressed_data, header, &blocks)?;

        for (index, section) in blocks.sections(&decompressed_data).enumerate() {
            debug!(
                r#"Simulating code execution for section {index} "{}""#,
                state.get_string(section.name.get())
            );
            if let Err(invalid_entry) = state.execute_code_segment(section.code.get()) {
                error!(%invalid_entry);
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
                let app_name = state.get_string(state.language_table.string_offsets[2].get());
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
                            position +=
                                data_offset + non_solid_start_offset as usize + size_of::<u32>();
                        }

                        let mut decoder = if is_solid {
                            solid_decoder
                        } else {
                            match compression {
                                Compression::Lzma(filter_flag) => {
                                    let mut data = &data[position + usize::from(filter_flag)..];
                                    let stream = LzmaStreamHeader::from_reader(&mut data).ok()?;
                                    Decoder::Lzma(XzDecoder::new_stream(data, stream))
                                }
                                Compression::BZip2 => {
                                    Decoder::BZip2(BzDecoder::new(&data[position..]))
                                }
                                Compression::Zlib => {
                                    Decoder::Zlib(ZlibDecoder::new_with_decompress(
                                        &data[position..],
                                        Decompress::new(false),
                                    ))
                                }
                                Compression::None => Decoder::None(&data[position..]),
                            }
                        };
                        let mut void = io::sink();

                        if is_solid {
                            // Seek to file
                            io::copy(&mut decoder.by_ref().take(position as u64), &mut void)
                                .ok()?;
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

        let display_name = state.registry.remove_value_by_name("DisplayName");
        let publisher = state.registry.remove_value_by_name("Publisher");
        let display_version = state.registry.remove_value_by_name("DisplayVersion");
        let product_code = state.registry.product_code();

        Ok(Self {
            installer: Installer {
                locale: Language::from_code(state.language_table.id.get())
                    .tag()
                    .parse::<LanguageTag>()
                    .ok(),
                architecture: architecture.unwrap_or(Architecture::X86),
                r#type: Some(InstallerType::Nullsoft),
                scope: state
                    .variables
                    .install_dir()
                    .and_then(Scope::from_install_directory),
                product_code: product_code.map(str::to_owned),
                apps_and_features_entries: if display_name.is_some()
                    || publisher.is_some()
                    || display_version.is_some()
                {
                    AppsAndFeaturesEntry::builder()
                        .maybe_display_name(display_name)
                        .maybe_publisher(publisher)
                        .maybe_display_version(display_version)
                        .maybe_product_code(product_code)
                        .build()
                        .into()
                } else {
                    AppsAndFeaturesEntries::new()
                },
                installation_metadata: InstallationMetadata {
                    default_install_location: state.variables.install_dir().map(Utf8PathBuf::from),
                    ..InstallationMetadata::default()
                },
                ..Installer::default()
            },
        })
    }
}
