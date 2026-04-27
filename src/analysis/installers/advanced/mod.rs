mod file_entry;
mod footer;
mod named_file_entry;

use std::{
    collections::BTreeSet,
    io::{self, Cursor, Read, Seek, SeekFrom},
};

use encoding_rs::UTF_16LE;
use file_entry::FileEntry;
use footer::Footer;
use named_file_entry::NamedFileEntry;
use sevenz_rust2::{ArchiveReader, Password};
use thiserror::Error;
use tracing::{debug, warn};
use winget_types::installer::{
    AppsAndFeaturesEntry, ExpectedReturnCodes, Installer, InstallerReturnCode, InstallerSwitches,
    InstallerType, ReturnResponse,
};
use zerocopy::IntoBytes;

use super::msi::Msi;
use crate::{
    analysis::{Installers, installers::pe::PE},
    read::ReadBytesExt,
};

#[derive(Error, Debug)]
pub enum AdvancedInstallerError {
    #[error("File is not an Advanced Installer")]
    NotAdvancedInstallerFile,
    #[error(transparent)]
    Io(#[from] io::Error),
}

pub struct AdvancedInstaller {
    installers: Vec<Msi>,
}

impl AdvancedInstaller {
    pub fn new<R: Read + Seek>(mut reader: R, pe: &PE) -> Result<Self, AdvancedInstallerError> {
        let end_offset = pe
            .certificate_table()
            .filter(|dir| dir.virtual_address() != 0)
            .map(|dir| dir.virtual_address().into())
            .or_else(|| pe.overlay_offset())
            .unwrap_or_else(|| reader.seek(SeekFrom::End(0)).unwrap_or_default());

        // 11-byte magic before EOF or cert table, padded to a to 4-byte boundary
        let search_start = end_offset.saturating_sub(18);
        reader.seek(SeekFrom::Start(search_start))?;

        let mut buf = vec![0; (end_offset - search_start) as usize];
        reader.read_exact(&mut buf)?;

        let signature_pos = buf
            .array_windows()
            .rposition(|window| window == Footer::SIGNATURE)
            .ok_or(AdvancedInstallerError::NotAdvancedInstallerFile)?;

        let footer_offset = search_start + signature_pos as u64 - Footer::SIGNATURE_OFFSET as u64;
        reader.seek(SeekFrom::Start(footer_offset))?;
        let footer = reader.read_t::<Footer>()?;

        debug!(?footer);

        reader.seek(SeekFrom::Start(footer.table_pointer().into()))?;

        let mut files = Vec::with_capacity(footer.num_files() as usize);
        for _ in 0..footer.num_files() {
            let file_entry = reader.read_t::<FileEntry>()?;

            let mut name_bytes = vec![0_u16; file_entry.name_size() as usize];
            reader.read_exact(name_bytes.as_mut_bytes())?;
            let name = UTF_16LE.decode(name_bytes.as_bytes()).0;

            let named_file_entry = NamedFileEntry::new(file_entry, name);
            debug!(?named_file_entry);
            files.push(named_file_entry);
        }

        // TODO are there cases where we should parse this ini?
        if let Some(ini_file) = files
            .iter()
            .rfind(|entry| entry.name().to_ascii_lowercase().ends_with(".ini"))
            && let Ok(ini_data) = ini_file.read_file(&mut reader)
        {
            debug!(ini = %UTF_16LE.decode(&ini_data).0);
        }

        let installers = files
            .iter()
            .rfind(|entry| entry.name().to_ascii_lowercase().ends_with(".7z"))
            .and_then(|archive| archive.read_file(&mut reader).ok())
            .and_then(|seven_z_data| {
                let mut msi_files = Vec::new();
                ArchiveReader::new(Cursor::new(&seven_z_data), Password::empty())
                    .ok()?
                    .for_each_entries(|entry, reader| {
                        debug!(seven_z_file = ?entry.name());
                        let mut buf = Vec::new();
                        if reader.read_to_end(&mut buf).is_ok()
                            && let Ok(msi) = Msi::new(Cursor::new(buf))
                        {
                            msi_files.push(msi);
                        }
                        Ok(true)
                    })
                    .ok()?;
                (!msi_files.is_empty()).then_some(msi_files)
            })
            .unwrap_or_else(|| {
                files
                    .iter()
                    .filter(|f| f.name().to_ascii_lowercase().ends_with(".msi"))
                    .filter_map(|msi_file| msi_file.read_file(&mut reader).ok())
                    .filter_map(|msi_data| Msi::new(Cursor::new(msi_data)).ok())
                    .collect()
            });

        if installers.is_empty() {
            warn!(
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
                installer.switches = InstallerSwitches::builder()
                    .silent("/exenoui /quiet".parse().unwrap())
                    .silent_with_progress("/exenoui /passive".parse().unwrap())
                    .install_location(r#"APPDIR="<INSTALLPATH>""#.parse().unwrap())
                    .log(r#"/log "<LOGPATH>""#.parse().unwrap())
                    .custom(
                        installer
                            .switches
                            .custom()
                            .cloned()
                            .map(|mut custom| {
                                custom.push("/norestart");
                                custom
                            })
                            .unwrap_or_else(|| "/norestart".parse().unwrap()),
                    )
                    .build();

                // https://www.advancedinstaller.com/user-guide/exe-setup-file.html#return-code
                installer.expected_return_codes = expected_return_codes();

                // If the MSI is hidden, there's another ARP entry that shares some values
                if msi
                    .property_table
                    .iter()
                    .any(|(key, value)| key == "ARPSYSTEMCOMPONENT" && value == "1")
                    && let Some(template) = installer.apps_and_features_entries.iter().next()
                {
                    let product_code = format!(
                        "{} {}",
                        template.display_name().unwrap_or_default(),
                        template.display_version().unwrap()
                    );
                    installer.product_code = Some(product_code.clone());
                    installer.apps_and_features_entries = AppsAndFeaturesEntry::builder()
                        .maybe_display_name(template.display_name())
                        .maybe_display_version(template.display_version().cloned())
                        .maybe_publisher(template.publisher())
                        .maybe_product_code(Some(product_code))
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
