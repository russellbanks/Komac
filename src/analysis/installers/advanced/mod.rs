mod file_entry;
mod footer;
mod named_file_entry;

use std::io::{self, Cursor, Read, Seek, SeekFrom};

use encoding_rs::UTF_16LE;
use file_entry::FileEntry;
use footer::Footer;
use named_file_entry::NamedFileEntry;
use sevenz_rust2::{ArchiveReader, Password};
use thiserror::Error;
use tracing::{debug, warn};
use winget_types::installer::{
    AppsAndFeaturesEntry, ExpectedReturnCode, Installer, InstallerReturnCode, InstallerType,
    ReturnResponse, Switches,
};
use zerocopy::IntoBytes;

use super::msi::Msi;
use crate::{analysis::Installers, read::ReadBytesExt};

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
    pub fn new<R: Read + Seek>(mut reader: R) -> Result<Self, AdvancedInstallerError> {
        let footer = Footer::find(&mut reader)?;

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

        if let Some(ini_file) = files.iter().rfind(|entry| entry.is_ini())
            && let Ok(ini_data) = ini_file.read_file(&mut reader)
        {
            debug!(ini = %UTF_16LE.decode(&ini_data).0);
        }

        let installers = files
            .iter()
            .rfind(|entry| entry.is_7z())
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
                    .filter(|entry| entry.is_msi())
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
                installer.switches = Switches::builder()
                    .silent("/exenoui /quiet".parse().unwrap())
                    .silent_with_progress("/exenoui /passive".parse().unwrap())
                    .install_location(r#"APPDIR="<INSTALLPATH>""#.parse().unwrap())
                    .log(r#"/log "<LOGPATH>""#.parse().unwrap())
                    .custom(installer.switches.custom().cloned().map_or_else(
                        || "/norestart".parse().unwrap(),
                        |mut custom| {
                            custom.push("/norestart");
                            custom
                        },
                    ))
                    .build();

                // https://www.advancedinstaller.com/user-guide/exe-setup-file.html#return-code
                installer.expected_return_codes = expected_return_codes().into();

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
                        .into();
                }

                installer
            })
            .collect()
    }
}

const fn expected_return_codes() -> [ExpectedReturnCode; 20] {
    use ReturnResponse::{
        AlreadyInstalled, BlockedByPolicy, CancelledByUser, ContactSupport, InstallInProgress,
        InvalidParameter, RebootInitiated, RebootRequiredToFinish, SystemNotSupported,
    };

    [
        ExpectedReturnCode::new(InstallerReturnCode::from_i32(-1).unwrap(), CancelledByUser),
        ExpectedReturnCode::new(InstallerReturnCode::from_u32(1).unwrap(), InvalidParameter),
        ExpectedReturnCode::new(InstallerReturnCode::from_u32(87).unwrap(), InvalidParameter),
        ExpectedReturnCode::new(InstallerReturnCode::from_u32(1601).unwrap(), ContactSupport),
        ExpectedReturnCode::new(
            InstallerReturnCode::from_u32(1602).unwrap(),
            CancelledByUser,
        ),
        ExpectedReturnCode::new(
            InstallerReturnCode::from_u32(1618).unwrap(),
            InstallInProgress,
        ),
        ExpectedReturnCode::new(
            InstallerReturnCode::from_u32(1623).unwrap(),
            SystemNotSupported,
        ),
        ExpectedReturnCode::new(
            InstallerReturnCode::from_u32(1625).unwrap(),
            BlockedByPolicy,
        ),
        ExpectedReturnCode::new(
            InstallerReturnCode::from_u32(1628).unwrap(),
            InvalidParameter,
        ),
        ExpectedReturnCode::new(
            InstallerReturnCode::from_u32(1633).unwrap(),
            SystemNotSupported,
        ),
        ExpectedReturnCode::new(
            InstallerReturnCode::from_u32(1638).unwrap(),
            AlreadyInstalled,
        ),
        ExpectedReturnCode::new(
            InstallerReturnCode::from_u32(1639).unwrap(),
            InvalidParameter,
        ),
        ExpectedReturnCode::new(
            InstallerReturnCode::from_u32(1640).unwrap(),
            BlockedByPolicy,
        ),
        ExpectedReturnCode::new(
            InstallerReturnCode::from_u32(1641).unwrap(),
            RebootInitiated,
        ),
        ExpectedReturnCode::new(
            InstallerReturnCode::from_u32(1643).unwrap(),
            BlockedByPolicy,
        ),
        ExpectedReturnCode::new(
            InstallerReturnCode::from_u32(1644).unwrap(),
            BlockedByPolicy,
        ),
        ExpectedReturnCode::new(
            InstallerReturnCode::from_u32(1649).unwrap(),
            BlockedByPolicy,
        ),
        ExpectedReturnCode::new(
            InstallerReturnCode::from_u32(1650).unwrap(),
            InvalidParameter,
        ),
        ExpectedReturnCode::new(
            InstallerReturnCode::from_u32(1654).unwrap(),
            SystemNotSupported,
        ),
        ExpectedReturnCode::new(
            InstallerReturnCode::from_u32(3010).unwrap(),
            RebootRequiredToFinish,
        ),
    ]
}
