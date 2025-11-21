mod manifest;
mod wix_burn_stub;

use std::{
    collections::HashMap,
    io,
    io::{Read, Seek, SeekFrom},
};

use cab::Cabinet;
use camino::Utf8PathBuf;
use manifest::{
    BurnManifest, Package, RelatedBundle, VariableType, install_condition::Value,
    package::msi::Provides,
};
use quick_xml::de::from_str;
use thiserror::Error;
use tracing::debug;
use winget_types::installer::{
    AppsAndFeaturesEntries, AppsAndFeaturesEntry, Architecture, InstallationMetadata, Installer,
    InstallerType, Scope,
};
use wix_burn_stub::WixBurnStub;

use super::msi::Msi;
use crate::{
    analysis::{
        Installers,
        installers::pe::{
            PE,
            resource::{ImageResourceDataEntry, ResourceDirectory, SectionReader},
        },
    },
    read::ReadBytesExt,
    traits::FromMachine,
};

#[derive(Error, Debug)]
pub enum BurnError {
    #[error("File is not a Burn installer")]
    NotBurnFile,
    #[error(transparent)]
    ManifestDeserialization(#[from] quick_xml::DeError),
    #[error(transparent)]
    Io(#[from] io::Error),
}

const WIXBURN: [u8; 8] = *b".wixburn";

pub struct Burn {
    architecture: Architecture,
    manifest: Option<BurnManifest>,
    msi: Option<Msi>,
}

impl Burn {
    pub fn new<R: Read + Seek>(mut reader: R, pe: &PE) -> Result<Self, BurnError> {
        let Some(wixburn) = pe.find_section(WIXBURN) else {
            // Get the resource table data directory header
            let resource_table = pe.resource_table().ok_or(BurnError::NotBurnFile)?;

            // Get the actual file offset of the resource directory section
            let resource_directory_offset = resource_table.file_offset(&pe.section_table)?;

            let section_reader = SectionReader::new(
                &mut reader,
                resource_directory_offset.into(),
                resource_table.size().into(),
            )?;

            let mut resource_directory = ResourceDirectory::new(section_reader)?;

            let _rc_data = resource_directory
                .find_rc_data()
                .map_err(|_| BurnError::NotBurnFile);

            let msi_entry = resource_directory
                .find_name_entry("MSI")
                .ok_or(BurnError::NotBurnFile)?;

            let msi_directory_table = msi_entry
                .data(&mut resource_directory)?
                .table()
                .ok_or(BurnError::NotBurnFile)?;

            return if let Some(msi_directory) = msi_directory_table.entries().next() {
                let msi_data_entry_offset = msi_directory.file_offset(resource_directory_offset);
                reader.seek(SeekFrom::Start(msi_data_entry_offset.into()))?;

                let msi_data_entry = reader.read_t::<ImageResourceDataEntry>()?;
                let msi_offset = pe
                    .section_table
                    .to_file_offset(msi_data_entry.offset_to_data())?;

                // Installers built with the Java Development Kit embed an MSI resource
                reader.seek(SeekFrom::Start(msi_offset.into()))?;
                let msi_reader = reader.take(msi_data_entry.size().into());
                let msi = Msi::new(msi_reader)?;
                Ok(Self {
                    architecture: msi.architecture,
                    manifest: None,
                    msi: Some(msi),
                })
            } else {
                Err(BurnError::NotBurnFile)
            };
        };

        // Seek to and read wix burn stub
        reader.seek(SeekFrom::Start(wixburn.pointer_to_raw_data().into()))?;
        let wix_burn_stub = WixBurnStub::try_read_from_io(&mut reader)?;

        debug!(?wix_burn_stub);

        // Read the UX container (contains installation logic, bundle manifest, layout, and
        // bootstrapper exe)
        reader.seek(SeekFrom::Start(wix_burn_stub.stub_size().into()))?;
        let mut ux_cabinet = Cabinet::new(
            reader.take(
                wix_burn_stub
                    .bootstrapper_application_container_size()
                    .into(),
            ),
        )?;

        // The Burn manifest is always file "0"
        let manifest = io::read_to_string(ux_cabinet.read_file("0")?)?;
        debug!(manifest);
        let manifest = from_str::<BurnManifest>(&manifest)?;
        debug!("{manifest:#?}");

        Ok(Self {
            architecture: if manifest.win_64 {
                Architecture::X64
            } else {
                Architecture::from_machine(pe.machine())
            },
            manifest: Some(manifest),
            msi: None,
        })
    }
}

impl Installers for Burn {
    fn installers(&self) -> Vec<Installer> {
        if let Some(ref msi) = self.msi {
            return msi.installers();
        }

        let manifest = self.manifest.as_ref().unwrap_or_else(|| unreachable!());

        let mut apps_and_features_entries = AppsAndFeaturesEntries::from(
            AppsAndFeaturesEntry::builder()
                .display_name(manifest.registration.arp.display_name())
                .maybe_publisher(manifest.registration.arp.publisher())
                .display_version(manifest.registration.arp.display_version().clone())
                .product_code(manifest.registration.id())
                .maybe_upgrade_code(manifest.related_bundles.first().map(RelatedBundle::code))
                .installer_type(InstallerType::Burn)
                .build(),
        );

        let variables = manifest
            .variables
            .iter()
            .filter_map(|variable| {
                let value = match variable.r#type {
                    VariableType::Numeric => Value::Int(variable.resolved_value()?.parse().ok()?),
                    VariableType::String => Value::Str(variable.resolved_value()?),
                    _ => return None,
                };

                Some((variable.id(), value))
            })
            .chain([
                ("VersionNT64", Value::Bool(true)),
                ("NativeMachine", Value::Int(0x8664)),
            ])
            .collect::<HashMap<_, _>>();

        for msi_package in manifest
            .chain
            .packages
            .iter()
            .filter_map(Package::try_as_msi)
            .filter(|msi_package| {
                // Even though it's still written to the registry, an `ARPSYSTEMCOMPONENT` value of
                // 1 prevents the application from being displayed in the Add or Remove Programs
                // list of Control Panel
                // https://learn.microsoft.com/windows/win32/msi/arpsystemcomponent
                !msi_package.is_arp_system_component()
            })
            .filter(|msi_package| msi_package.evaluate_install_condition(&variables))
        {
            apps_and_features_entries.push(
                AppsAndFeaturesEntry::builder()
                    .maybe_display_name(
                        msi_package.provides.iter().find_map(Provides::display_name),
                    )
                    .maybe_publisher(manifest.registration.arp.publisher())
                    .display_version(msi_package.version().clone())
                    .product_code(msi_package.product_code())
                    .maybe_upgrade_code(msi_package.upgrade_code())
                    .installer_type(
                        if manifest.payloads.iter().any(|payload| {
                            payload.id() == msi_package.id()
                                && payload
                                    .container
                                    .as_deref()
                                    .is_some_and(|container| container.starts_with("Wix"))
                        }) {
                            InstallerType::Wix
                        } else {
                            InstallerType::Msi
                        },
                    )
                    .build(),
            );
        }

        vec![Installer {
            architecture: self.architecture,
            r#type: Some(InstallerType::Burn),
            scope: manifest
                .registration
                .per_machine
                .then_some(Scope::Machine)
                .or(Some(Scope::User)),
            apps_and_features_entries,
            installation_metadata: manifest
                .variables
                .iter()
                .find_map(|variable| {
                    (variable.id() == "InstallFolder").then(|| variable.resolved_value())?
                })
                .filter(|value| !value.contains(['[', ']']))
                .map(|install_folder| InstallationMetadata {
                    default_install_location: Some(Utf8PathBuf::from(&install_folder)),
                    ..InstallationMetadata::default()
                })
                .unwrap_or_default(),
            ..Installer::default()
        }]
    }
}
