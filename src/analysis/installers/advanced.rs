use std::{
    collections::BTreeSet,
    io::{Cursor, Read, Seek, SeekFrom},
};

use byteorder::{LittleEndian, ReadBytesExt};
use encoding_rs::UTF_16LE;
use sevenz_rust2::{ArchiveReader, Password};
use thiserror::Error;
use tracing::debug;
use winget_types::installer::{
    AppsAndFeaturesEntry, ExpectedReturnCodes, Installer, InstallerReturnCode, InstallerSwitches,
    InstallerType, ReturnResponse,
};
use yara_x::mods::PE;

use crate::analysis::Installers;

use super::msi::Msi;

#[derive(Error, Debug)]
pub enum AdvancedInstallerError {
    #[error("File is not an Advanced Installer")]
    NotAdvancedInstallerFile,
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub struct AdvancedInstaller {
    installers: Vec<Msi>,
}

impl AdvancedInstaller {
    pub fn new<R: Read + Seek>(mut reader: R, pe: &PE) -> Result<Self, AdvancedInstallerError> {
        let end_offset = pe
            .data_directories
            .get(4)
            .filter(|dir| dir.virtual_address() != 0)
            .map(|dir| dir.virtual_address() as u64)
            .unwrap_or_else(|| reader.seek(SeekFrom::End(0)).unwrap_or(0));

        // 11-byte magic before EOF or cert table, padded with trailing nulls to 8-byte boundary
        let search_start = end_offset.saturating_sub(18);
        reader.seek(SeekFrom::Start(search_start))?;
        let mut buf = vec![0u8; (end_offset - search_start) as usize];
        reader.read_exact(&mut buf)?;
        let magic_pos = buf
            .windows(11)
            .rposition(|w| w == b"ADVINSTSFX\0")
            .ok_or(AdvancedInstallerError::NotAdvancedInstallerFile)?;

        let footer_offset = search_start + magic_pos as u64 - 60;
        reader.seek(SeekFrom::Start(footer_offset + 4))?;
        let num_files = reader.read_i32::<LittleEndian>()?;
        reader.seek(SeekFrom::Current(8))?;
        let files_offset = reader.read_i32::<LittleEndian>()?;

        reader.seek(SeekFrom::Start(files_offset as u64))?;
        let mut files = Vec::with_capacity(num_files as usize);
        for _ in 0..num_files {
            reader.seek(SeekFrom::Current(8))?;
            let encoding_flag = reader.read_i32::<LittleEndian>()? as u32;
            let size = reader.read_i32::<LittleEndian>()? as u32;
            let offset = reader.read_i32::<LittleEndian>()? as u32;
            let name_chars = reader.read_i32::<LittleEndian>()? as usize;
            let mut name_bytes = vec![0u8; name_chars * 2];
            reader.read_exact(&mut name_bytes)?;
            let name = UTF_16LE.decode(&name_bytes).0.trim_matches('\0').to_owned();
            debug!(file = ?name);
            files.push(File {
                name,
                size,
                offset,
                encoding_flag,
            });
        }

        // TODO are there cases where we should parse this ini?
        if let Some(ini_file) = files
            .iter()
            .rev()
            .find(|f| f.name.to_ascii_lowercase().ends_with(".ini"))
            && let Ok(ini_data) = ini_file.read(&mut reader)
        {
            debug!(ini = %UTF_16LE.decode(&ini_data).0);
        }

        let installers = files
            .iter()
            .rev()
            .find(|f| f.name.to_ascii_lowercase().ends_with(".7z"))
            .and_then(|archive| archive.read(&mut reader).ok())
            .and_then(|seven_z_data| {
                let mut msis = Vec::new();
                ArchiveReader::new(Cursor::new(&seven_z_data), Password::empty())
                    .ok()?
                    .for_each_entries(|_entry, reader| {
                        debug!(seven_z_file = ?_entry.name());
                        let mut buf = Vec::new();
                        reader.read_to_end(&mut buf)?;
                        if let Ok(msi) = Msi::new(Cursor::new(buf)) {
                            msis.push(msi);
                        }
                        Ok(true)
                    })
                    .ok()?;
                (!msis.is_empty()).then_some(msis)
            })
            .unwrap_or_else(|| {
                files
                    .iter()
                    .filter(|f| f.name.to_ascii_lowercase().ends_with(".msi"))
                    .filter_map(|msi_file| msi_file.read(&mut reader).ok())
                    .filter_map(|msi_data| Msi::new(Cursor::new(msi_data)).ok())
                    .collect()
            });

        if installers.is_empty() {
            tracing::warn!(
                "Detected Advanced Installer with no MSI files. Please open an issue: https://github.com/russellbanks/Komac/issues/new?template=bug.yml"
            );
            return Err(AdvancedInstallerError::NotAdvancedInstallerFile);
        }

        Ok(Self { installers })
    }
}

impl Installers for AdvancedInstaller {
    fn installers(&self) -> Vec<Installer> {
        self.installers
            .iter()
            .map(|msi| {
                let mut installer = msi.installers().into_iter().next().unwrap_or_default();
                installer.r#type = Some(InstallerType::Exe);

                // https://www.advancedinstaller.com/user-guide/exe-setup-file.html#proprietary-command-line-switches-for-the-exe-setup
                let custom_switches = installer
                    .switches
                    .custom()
                    .map(|s| format!("/norestart {}", s))
                    .unwrap_or_else(|| "/norestart".to_string());
                installer.switches = InstallerSwitches::builder()
                    .silent("/exenoui /quiet".parse().unwrap())
                    .silent_with_progress("/exenoui /passive".parse().unwrap())
                    .install_location("APPDIR=\"<INSTALLPATH>\"".parse().unwrap())
                    .log("/log \"<LOGPATH>\"".parse().unwrap())
                    .custom(custom_switches.parse().unwrap())
                    .build();

                // https://www.advancedinstaller.com/user-guide/exe-setup-file.html#return-code
                installer.expected_return_codes = expected_return_codes();

                // If the MSI is hidden, there's another ARP entry that shares some values
                let hidden = msi
                    .property_table
                    .iter()
                    .find(|(key, _)| *key == "ARPSYSTEMCOMPONENT")
                    .is_some_and(|(_, value)| value == "1");
                if let Some(template) = installer.apps_and_features_entries.iter().next()
                    && hidden
                {
                    installer.product_code = Some(format!(
                        "{} {}",
                        template.display_name().unwrap_or_default(),
                        template.display_version().unwrap()
                    ));
                    installer.apps_and_features_entries = AppsAndFeaturesEntry::builder()
                        .maybe_display_name(template.display_name())
                        .maybe_display_version(template.display_version().cloned())
                        .maybe_publisher(template.publisher())
                        .maybe_product_code(installer.product_code.clone())
                        .build()
                        .into()
                }

                installer
            })
            .collect()
    }
}

fn expected_return_codes() -> BTreeSet<ExpectedReturnCodes> {
    use ReturnResponse::*;
    [
        (-1, CancelledByUser),
        (1, InvalidParameter),
        (87, InvalidParameter),
        (1601, ContactSupport),
        (1602, CancelledByUser),
        (1618, InstallInProgress),
        (1623, SystemNotSupported),
        (1625, BlockedByPolicy),
        (1628, InvalidParameter),
        (1633, SystemNotSupported),
        (1638, AlreadyInstalled),
        (1639, InvalidParameter),
        (1640, BlockedByPolicy),
        (1641, RebootInitiated),
        (1643, BlockedByPolicy),
        (1644, BlockedByPolicy),
        (1649, BlockedByPolicy),
        (1650, InvalidParameter),
        (1654, SystemNotSupported),
        (3010, RebootRequiredToFinish),
    ]
    .into_iter()
    .map(|(code, response)| ExpectedReturnCodes {
        installer_return_code: InstallerReturnCode::new(code),
        return_response: response,
        return_response_url: None,
    })
    .collect()
}

struct File {
    name: String,
    size: u32,
    offset: u32,
    encoding_flag: u32,
}

impl File {
    fn read<R: Read + Seek>(&self, reader: &mut R) -> std::io::Result<Vec<u8>> {
        reader.seek(SeekFrom::Start(self.offset.into()))?;
        let mut data = vec![0u8; self.size as usize];
        reader.read_exact(&mut data)?;
        if self.encoding_flag == 2 {
            data.iter_mut().take(0x200).for_each(|b| *b ^= 0xFF);
        }
        Ok(data)
    }
}
