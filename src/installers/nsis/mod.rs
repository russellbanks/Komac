mod entry;
mod first_header;
mod header;
mod language;
mod state;
mod strings;
mod version;

use crate::file_analyser::EXE;
use crate::installers::nsis::entry::Entry;
use crate::installers::nsis::first_header::FirstHeader;
use crate::installers::nsis::header::block::BlockHeaders;
use crate::installers::nsis::header::compression::Compression;
use crate::installers::nsis::header::flags::CommonHeaderFlags;
use crate::installers::nsis::header::{Decompressed, Header};
use crate::installers::utils::{read_lzma_stream_header, RELATIVE_PROGRAM_FILES_64};
use crate::manifests::installer_manifest::{
    AppsAndFeaturesEntry, InstallationMetadata, Installer, Scope,
};
use crate::types::architecture::Architecture;
use crate::types::installer_type::InstallerType;
use crate::types::language_tag::LanguageTag;
use crate::types::version::Version;
use byteorder::{ReadBytesExt, LE};
use bzip2::read::BzDecoder;
use camino::{Utf8Path, Utf8PathBuf};
use flate2::read::DeflateDecoder;
use header::block::BlockType;
use liblzma::read::XzDecoder;
use msi::Language;
use protobuf::Enum;
use state::NsisState;
use std::borrow::Cow;
use std::io;
use std::io::Read;
use strsim::levenshtein;
use thiserror::Error;
use yara_x::mods::pe::Machine;
use yara_x::mods::PE;
use zerocopy::little_endian::I32;
use zerocopy::{FromBytes, TryFromBytes};

#[derive(Error, Debug)]
pub enum NsisError {
    #[error("File is not a NSIS installer")]
    NotNsisFile,
    #[error("Failed to get NSIS first header offset")]
    FirstHeaderOffset,
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

        let mut state = NsisState::new(pe, &decompressed_data, header, &blocks)?;

        let entries =
            <[Entry]>::try_ref_from_bytes(BlockType::Entries.get(&decompressed_data, &blocks))
                .map_err(|error| NsisError::ZeroCopy(error.to_string()))?;

        let mut architecture =
            Option::from(architecture).filter(|&architecture| architecture != Architecture::X86);

        let mut display_name = None;
        let mut display_version = None;
        let mut display_publisher = None;
        for entry in entries {
            entry.update_vars(&mut state);
            if let Entry::WriteReg {
                value_name, value, ..
            } = entry
            {
                let value = state.get_string(value.get());
                match &*state.get_string(value_name.get()) {
                    "DisplayName" => display_name = Some(value),
                    "DisplayVersion" => display_version = Some(value),
                    "Publisher" => display_publisher = Some(value),
                    _ => {}
                }
            } else if let Entry::ExtractFile { name, .. } = entry {
                let name = state.get_string(name.get());
                let file_stem = Utf8Path::new(&name).file_stem();
                // If there is an app-64 file, the app is x64.
                // If there is an app-32 file or both files are present, the app is x86
                // (x86 apps can still install on x64 systems)
                if file_stem == Some(APP_64) && architecture.is_none() {
                    architecture = Some(Architecture::X64);
                } else if file_stem == Some(APP_32) {
                    architecture = Some(Architecture::X86);
                }
            };
        }

        let install_dir = (header.install_directory_ptr != I32::ZERO)
            .then(|| state.get_string(header.install_directory_ptr.get()));

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
                        let mut decoder: Box<dyn Read> = if is_solid {
                            solid_decoder
                        } else {
                            match compression {
                                Compression::Lzma(filter_flag) => {
                                    let mut data = &data[position + usize::from(filter_flag)..];
                                    let stream = read_lzma_stream_header(&mut data).ok()?;
                                    Box::new(XzDecoder::new_stream(data, stream))
                                }
                                Compression::BZip2 => Box::new(BzDecoder::new(&data[position..])),
                                Compression::Zlib => {
                                    Box::new(DeflateDecoder::new(&data[position..]))
                                }
                                Compression::None => Box::new(&data[position..]),
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

                        let machine_value = decoder.read_u16::<LE>().ok()?;
                        Machine::from_i32(i32::from(machine_value))
                    })
                    .map(Architecture::from_machine)
            });

        Ok(Self {
            installer: Installer {
                locale: Language::from_code(state.language_table.id.get())
                    .tag()
                    .parse::<LanguageTag>()
                    .ok(),
                architecture: architecture.unwrap_or(Architecture::X86),
                r#type: Some(InstallerType::Nullsoft),
                scope: install_dir.as_deref().and_then(Scope::from_install_dir),
                apps_and_features_entries: [&display_name, &display_version, &display_publisher]
                    .iter()
                    .any(|option| option.is_some())
                    .then(|| {
                        vec![AppsAndFeaturesEntry {
                            display_name: display_name.map(Cow::into_owned),
                            publisher: display_publisher.map(Cow::into_owned),
                            display_version: display_version.as_deref().map(Version::new),
                            ..AppsAndFeaturesEntry::default()
                        }]
                    }),
                installation_metadata: install_dir.is_some().then(|| InstallationMetadata {
                    default_install_location: install_dir.as_deref().map(Utf8PathBuf::from),
                    ..InstallationMetadata::default()
                }),
                ..Installer::default()
            },
        })
    }
}
