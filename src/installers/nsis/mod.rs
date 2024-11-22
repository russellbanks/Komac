mod entry;
mod first_header;
mod header;
mod language;
mod strings;
mod version;

use crate::file_analyser::EXE;
use crate::installers::nsis::entry::file::ExtractFile;
use crate::installers::nsis::entry::registry::WriteReg;
use crate::installers::nsis::entry::Entry;
use crate::installers::nsis::first_header::FirstHeader;
use crate::installers::nsis::header::compression::Compression;
use crate::installers::nsis::header::{Decompressed, Header};
use crate::installers::nsis::language::table::LanguageTable;
use crate::installers::nsis::version::NsisVersion;
use crate::installers::traits::InstallSpec;
use crate::installers::utils::{read_lzma_stream_header, RELATIVE_PROGRAM_FILES_64};
use crate::manifests::installer_manifest::Scope;
use crate::types::architecture::Architecture;
use crate::types::installer_type::InstallerType;
use crate::types::language_tag::LanguageTag;
use byteorder::{ReadBytesExt, LE};
use bzip2::read::BzDecoder;
use camino::{Utf8Path, Utf8PathBuf};
use flate2::read::DeflateDecoder;
use header::block::BlockType;
use liblzma::read::XzDecoder;
use msi::Language;
use protobuf::Enum;
use std::io::Read;
use std::str::FromStr;
use std::{io, mem};
use strings::encoding::nsis_string;
use strsim::levenshtein;
use thiserror::Error;
use yara_x::mods::pe::Machine;
use yara_x::mods::PE;
use zerocopy::little_endian::U32;
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

pub struct Nsis {
    architecture: Architecture,
    scope: Option<Scope>,
    install_dir: Option<Utf8PathBuf>,
    install_locale: Option<LanguageTag>,
    display_name: Option<String>,
    display_version: Option<String>,
    display_publisher: Option<String>,
}

impl Nsis {
    pub fn new(data: &[u8], pe: &PE) -> Result<Self, NsisError> {
        let first_header_offset = pe
            .overlay
            .offset
            .and_then(|offset| usize::try_from(offset).ok())
            .ok_or(NsisError::FirstHeaderOffset)?;

        let data_offset = first_header_offset + size_of::<FirstHeader>();
        let first_header = FirstHeader::try_ref_from_bytes(&data[first_header_offset..data_offset])
            .map_err(|_| NsisError::NotNsisFile)?;

        let Decompressed {
            decompressed_data,
            is_solid,
            non_solid_start_offset,
            compression,
            decoder: solid_decoder,
        } = Header::decompress(&data[data_offset..], first_header)?;
        let (header, _) = Header::ref_from_prefix(&decompressed_data)
            .map_err(|error| NsisError::ZeroCopy(error.to_string()))?;

        let strings_block = BlockType::Strings.get(&decompressed_data, &header.blocks);

        let language_table = LanguageTable::get_main(&decompressed_data, header)?;

        let nsis_version = NsisVersion::from_manifest(data, pe)
            .or_else(|| NsisVersion::from_branding_text(strings_block, language_table))
            .unwrap_or_else(|| NsisVersion::detect(strings_block));

        let entries = <[Entry]>::try_ref_from_bytes(
            BlockType::Entries.get(&decompressed_data, &header.blocks),
        )
        .map_err(|error| NsisError::ZeroCopy(error.to_string()))?;

        let install_dir = (header.install_directory_ptr != U32::ZERO).then(|| {
            nsis_string(
                strings_block,
                header.install_directory_ptr.get(),
                entries,
                nsis_version,
            )
        });

        let app_name = nsis_string(
            strings_block,
            language_table.language_string_offsets[2].get(),
            entries,
            nsis_version,
        );

        let architecture = entries
            .iter()
            .filter_map(|entry| ExtractFile::from_entry(entry, strings_block, nsis_version))
            .filter(|file| {
                Utf8Path::new(&file.name)
                    .extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case(EXE))
            })
            .min_by_key(|file| levenshtein(&file.name, &app_name))
            .map(|mut file| {
                if !is_solid {
                    file.position +=
                        data_offset + non_solid_start_offset as usize + size_of::<u32>();
                }
                file
            })
            .and_then(|file| {
                let mut decoder: Box<dyn Read> = if is_solid {
                    solid_decoder
                } else {
                    match compression {
                        Compression::Lzma(filter_flag) => {
                            let mut data = &data[file.position + usize::from(filter_flag)..];
                            let stream = read_lzma_stream_header(&mut data).ok()?;
                            Box::new(XzDecoder::new_stream(data, stream))
                        }
                        Compression::BZip2 => Box::new(BzDecoder::new(&data[file.position..])),
                        Compression::Zlib => Box::new(DeflateDecoder::new(&data[file.position..])),
                        Compression::None => Box::new(&data[file.position..]),
                    }
                };
                let mut void = io::sink();

                if is_solid {
                    // Seek to file
                    io::copy(&mut decoder.by_ref().take(file.position as u64), &mut void).ok()?;
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
            .and_then(|machine| Architecture::from_machine(machine).ok())
            .unwrap_or_else(|| {
                if install_dir
                    .as_deref()
                    .is_some_and(|dir| dir.contains(RELATIVE_PROGRAM_FILES_64))
                {
                    Architecture::X64
                } else {
                    Architecture::X86
                }
            });

        let mut write_reg = entries
            .iter()
            .filter_map(|entry| WriteReg::from_entry(entry, strings_block, entries, nsis_version))
            .collect::<Vec<_>>();

        Ok(Self {
            architecture,
            scope: install_dir.as_deref().and_then(Scope::from_install_dir),
            install_dir: install_dir.as_deref().map(Utf8PathBuf::from),
            install_locale: LanguageTag::from_str(
                Language::from_code(language_table.language_id.get()).tag(),
            )
            .ok(),
            display_name: write_reg
                .iter_mut()
                .find(|write_reg| write_reg.value_name == "DisplayName")
                .map(|write_reg| mem::take(&mut write_reg.value).into_owned()),
            display_version: write_reg
                .iter_mut()
                .find(|write_reg| write_reg.value_name == "DisplayVersion")
                .map(|write_reg| mem::take(&mut write_reg.value).into_owned()),
            display_publisher: write_reg
                .iter_mut()
                .find(|write_reg| write_reg.value_name == "Publisher")
                .map(|write_reg| mem::take(&mut write_reg.value).into_owned()),
        })
    }
}

impl InstallSpec for Nsis {
    fn r#type(&self) -> InstallerType {
        InstallerType::Nullsoft
    }

    fn architecture(&mut self) -> Option<Architecture> {
        Some(self.architecture)
    }

    fn display_name(&mut self) -> Option<String> {
        self.display_name.take()
    }

    fn display_publisher(&mut self) -> Option<String> {
        self.display_publisher.take()
    }

    fn display_version(&mut self) -> Option<String> {
        self.display_version.take()
    }

    fn locale(&mut self) -> Option<LanguageTag> {
        self.install_locale.take()
    }

    fn scope(&self) -> Option<Scope> {
        self.scope
    }

    fn install_location(&mut self) -> Option<Utf8PathBuf> {
        self.install_dir.take()
    }
}
