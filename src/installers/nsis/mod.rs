mod entry;
mod file_system;
mod first_header;
mod header;
mod language;
mod registry;
mod section;
mod state;
mod strings;
mod version;

use std::{io, io::Read};

use byteorder::{LE, ReadBytesExt};
use bzip2::read::BzDecoder;
use camino::{Utf8Path, Utf8PathBuf};
use compact_str::CompactString;
use flate2::read::DeflateDecoder;
use header::block::BlockType;
use liblzma::read::XzDecoder;
use msi::Language;
use protobuf::Enum;
use state::NsisState;
use strsim::levenshtein;
use thiserror::Error;
use tracing::debug;
use winget_types::{
    LanguageTag, Version,
    installer::{
        AppsAndFeaturesEntry, Architecture, InstallationMetadata, Installer, InstallerType, Scope,
    },
};
use yara_x::mods::{PE, pe::Machine};
use zerocopy::{FromBytes, TryFromBytes, little_endian::I32};

use crate::{
    file_analyser::EXE,
    installers::{
        nsis::{
            entry::{Entry, EntryError},
            file_system::Directory,
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

        debug!(first_header_offset, ?first_header);

        let Decompressed {
            data: decompressed_data,
            is_solid,
            non_solid_start_offset,
            compression,
            decoder: solid_decoder,
        } = Header::decompress(&data[data_offset..], first_header)?;

        let architecture = Architecture::from_machine(pe.machine());

        let (_flags, rest) = CommonHeaderFlags::ref_from_prefix(&decompressed_data)
            .map_err(|error| NsisError::ZeroCopy(error.to_string()))?;

        let (blocks, rest) = BlockHeaders::read_dynamic_from_prefix(rest, architecture)?;

        let (header, _) = Header::ref_from_prefix(rest)
            .map_err(|error| NsisError::ZeroCopy(error.to_string()))?;

        debug!(?header);

        let mut state = NsisState::new(pe, &decompressed_data, header, &blocks)?;

        let install_dir = (header.install_directory_ptr != I32::ZERO)
            .then(|| state.get_string(header.install_directory_ptr.get()));

        if let Some(ref install_dir) = install_dir {
            debug!(%install_dir);
        }

        let entries =
            <[Entry]>::try_ref_from_bytes(BlockType::Entries.get(&decompressed_data, &blocks))
                .map_err(|error| NsisError::ZeroCopy(error.to_string()))?;

        for (index, section) in blocks.sections(&decompressed_data).enumerate() {
            debug!(
                r#"Simulating code execution for section {index} "{}""#,
                state.get_string(section.name.get())
            );
            state.execute_code_segment(section.code.get())?;
        }

        let mut architecture =
            Option::from(architecture).filter(|&architecture| architecture != Architecture::X86);

        for directory in state.file_system.directories().map(Directory::name) {
            // If there is an app-64 file, the app is x64.
            // If there is an app-32 file or both files are present, the app is x86
            // (x86 apps can still install on x64 systems)
            if directory == APP_64 && architecture.is_none() {
                architecture = Some(Architecture::X64);
            } else if directory == APP_32 {
                architecture = Some(Architecture::X86);
            }
        }

        architecture = architecture
            .or_else(|| {
                install_dir
                    .as_deref()
                    .is_some_and(|dir| dir.contains(RELATIVE_PROGRAM_FILES_64))
                    .then_some(Architecture::X64)
            })
            .or_else(|| {
                let app_name = state.get_string(state.language_table.string_offsets[2].get());
                entries
                    .iter()
                    .filter_map(|entry| {
                        if let Entry::ExtractFile { name, position, .. } = entry {
                            Some((
                                state.get_string(name.get()),
                                position.get().unsigned_abs() as usize + size_of::<u32>(),
                            ))
                        } else {
                            None
                        }
                    })
                    .filter(|(name, _)| {
                        Utf8Path::new(name)
                            .extension()
                            .is_some_and(|extension| extension.eq_ignore_ascii_case(EXE))
                    })
                    .min_by_key(|(name, _)| levenshtein(name, &app_name))
                    .map(|(_, mut position)| {
                        if !is_solid {
                            position +=
                                data_offset + non_solid_start_offset as usize + size_of::<u32>();
                        }
                        position
                    })
                    .and_then(|position| {
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
                                    Decoder::Zlib(DeflateDecoder::new(&data[position..]))
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
                scope: install_dir
                    .as_deref()
                    .and_then(Scope::from_install_directory),
                product_code: product_code.map(str::to_owned),
                apps_and_features_entries: if display_name.is_some()
                    || publisher.is_some()
                    || display_version.is_some()
                {
                    vec![AppsAndFeaturesEntry {
                        display_name: display_name.map(CompactString::from),
                        publisher: publisher.map(CompactString::from),
                        display_version: display_version.as_deref().map(Version::new),
                        product_code: product_code.map(str::to_owned),
                        ..AppsAndFeaturesEntry::default()
                    }]
                } else {
                    vec![]
                },
                installation_metadata: InstallationMetadata {
                    default_install_location: install_dir.as_deref().map(Utf8PathBuf::from),
                    ..InstallationMetadata::default()
                },
                ..Installer::default()
            },
        })
    }
}
