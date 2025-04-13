mod manifest;
mod wix_burn_stub;

use std::{io, io::Cursor};

use cab::Cabinet;
use camino::Utf8PathBuf;
use compact_str::CompactString;
use quick_xml::de::from_str;
use thiserror::Error;
use tracing::debug;
use winget_types::installer::{
    AppsAndFeaturesEntry, Architecture, InstallationMetadata, Installer, InstallerType, Scope,
};
use yara_x::mods::{
    PE,
    pe::{Resource, ResourceType, Section},
};
use zerocopy::TryFromBytes;

use crate::{
    installers::{
        burn::{
            manifest::{BurnManifest, Package},
            wix_burn_stub::WixBurnStub,
        },
        msi::Msi,
    },
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

pub struct Burn {
    pub installer: Installer,
}

impl Burn {
    pub fn new(data: &[u8], pe: &PE) -> Result<Self, BurnError> {
        if let Some(stub) = Self::get_wixburn_section(pe).and_then(|section| {
            let offset = section.raw_data_offset() as usize;
            data.get(offset..offset + size_of::<WixBurnStub>())
                .and_then(|section_bytes| WixBurnStub::try_ref_from_bytes(section_bytes).ok())
        }) {
            debug!(?stub);

            // UX container (contains installation logic, bundle manifest, layout, and bootstrapper exe)
            let ux_cabinet = &data[stub.ux_container_slice_range()];
            let mut ux_cabinet = Cabinet::new(Cursor::new(ux_cabinet))?;

            let manifest = io::read_to_string(ux_cabinet.read_file("0")?)?;
            let manifest = from_str::<BurnManifest>(&manifest)?;

            let mut apps_and_features_entries = vec![
                AppsAndFeaturesEntry::new()
                    .with_display_name(manifest.registration.arp.display_name)
                    .with_publisher::<_, &str>(manifest.registration.arp.publisher)
                    .with_display_version(manifest.registration.arp.display_version)
                    .with_product_code(manifest.registration.id)
                    .with_upgrade_code(manifest.related_bundle.code)
                    .with_installer_type(InstallerType::Burn),
            ];

            for msi_package in manifest
                .chain
                .packages
                .into_iter()
                .filter_map(|package| match package {
                    Package::Msi(msi_package) => Some(msi_package),
                    _ => None,
                })
                .filter(|msi_package| {
                    // Even though it's still written to the registry, an `ARPSYSTEMCOMPONENT` value
                    // of 1 prevents the application from being displayed in the Add or Remove
                    // Programs list of Control Panel
                    // https://learn.microsoft.com/windows/win32/msi/arpsystemcomponent
                    msi_package
                        .properties
                        .iter()
                        .find(|property| property.is_arp_system_component())
                        .is_none_or(|property| property.value.parse() != Ok(1))
                })
            {
                apps_and_features_entries.push(AppsAndFeaturesEntry {
                    display_name: msi_package.provides.display_name.map(CompactString::from),
                    publisher: manifest.registration.arp.publisher.map(CompactString::from),
                    display_version: Some(msi_package.version),
                    product_code: Some(manifest.registration.id.to_owned()),
                    upgrade_code: msi_package.upgrade_code.map(str::to_owned),
                    installer_type: manifest
                        .payloads
                        .iter()
                        .any(|payload| {
                            payload.id == msi_package.id
                                && payload
                                    .container
                                    .is_some_and(|container| container.starts_with("Wix"))
                        })
                        .then_some(InstallerType::Wix)
                        .or(Some(InstallerType::Msi)),
                });
            }

            Ok(Self {
                installer: Installer {
                    architecture: manifest
                        .win_64
                        .then_some(Architecture::X64)
                        .unwrap_or_else(|| Architecture::from_machine(pe.machine())),
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
                            (variable.id == "InstallFolder").then(|| variable.resolved_value())?
                        })
                        .filter(|value| !value.contains(['[', ']']))
                        .map(|install_folder| InstallationMetadata {
                            default_install_location: Some(Utf8PathBuf::from(&install_folder)),
                            ..InstallationMetadata::default()
                        })
                        .unwrap_or_default(),
                    ..Installer::default()
                },
            })
        } else if let Some(msi_resource) = Self::get_msi_resource(pe) {
            // Installers built with the Java Development Kit embed an MSI resource
            let offset = msi_resource.offset() as usize;
            let data = &data[offset..offset + msi_resource.length() as usize];
            Ok(Self {
                installer: Installer {
                    r#type: Some(InstallerType::Burn),
                    ..Msi::new(Cursor::new(data))?.installer
                },
            })
        } else {
            Err(BurnError::NotBurnFile)
        }
    }

    fn get_wixburn_section(pe: &PE) -> Option<&Section> {
        const WIXBURN_HEADER: &[u8] = b".wixburn";

        pe.sections
            .iter()
            .find(|section| section.name() == WIXBURN_HEADER)
    }

    fn get_msi_resource(pe: &PE) -> Option<&Resource> {
        const MSI: &[u8] = b"M\0S\0I\0";

        pe.resources
            .iter()
            .filter(|resource| resource.type_() == ResourceType::RESOURCE_TYPE_RCDATA)
            .find(|resource| resource.name_string() == MSI)
    }
}
