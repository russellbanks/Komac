pub mod bundle;
mod utils;

use crate::file_analyser::MSIX;
use crate::installers::msix_family::utils::{get_install_location, hash_signature, read_manifest};
use crate::manifests::installer_manifest::{
    AppsAndFeaturesEntry, InstallationMetadata, Installer, Platform, UpgradeBehavior,
};
use crate::types::architecture::Architecture;
use crate::types::file_extension::FileExtension;
use crate::types::installer_type::InstallerType;
use crate::types::minimum_os_version::MinimumOSVersion;
use crate::types::version::Version;
use color_eyre::eyre::Result;
use package_family_name::PackageFamilyName;
use quick_xml::events::Event;
use quick_xml::Reader;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::io::{Read, Seek};
use std::str::FromStr;
use zip::ZipArchive;

pub struct Msix {
    pub installer: Installer,
}

const APPX_MANIFEST_XML: &str = "AppxManifest.xml";
pub const APPX_SIGNATURE_P7X: &str = "AppxSignature.p7x";

const MSIX_MIN_VERSION: MinimumOSVersion = MinimumOSVersion(10, 0, 17763, 0);

impl Msix {
    pub fn new<R: Read + Seek>(reader: R) -> Result<Self> {
        let mut zip = ZipArchive::new(reader)?;

        let mut appx_manifest = read_manifest(&mut zip, APPX_MANIFEST_XML)?;

        let signature_sha_256 = hash_signature(&mut zip)?;

        let mut manifest = Package::default();

        let mut reader = Reader::from_str(&appx_manifest);
        let config = reader.config_mut();
        config.expand_empty_elements = true;
        config.trim_text(true);

        loop {
            match reader.read_event()? {
                Event::Start(event) => match event.local_name().as_ref() {
                    b"Identity" => {
                        for attribute in event.attributes().flatten() {
                            match attribute.key.as_ref() {
                                b"Name" => {
                                    manifest.identity.name =
                                        String::from_utf8_lossy(&attribute.value).into_owned();
                                }
                                b"Version" => {
                                    manifest.identity.version =
                                        String::from_utf8_lossy(&attribute.value).into_owned();
                                }
                                b"Publisher" => {
                                    manifest.identity.publisher =
                                        String::from_utf8_lossy(&attribute.value).into_owned();
                                }
                                b"ProcessorArchitecture" => {
                                    manifest.identity.processor_architecture =
                                        String::from_utf8_lossy(&attribute.value).into_owned();
                                }
                                b"ResourceId" => {
                                    manifest.identity.resource_id =
                                        String::from_utf8_lossy(&attribute.value).into_owned();
                                }
                                _ => continue,
                            }
                        }
                    }
                    b"DisplayName" => {
                        manifest.properties.display_name =
                            reader.read_text(event.to_end().name())?.into_owned();
                    }
                    b"PublisherDisplayName" => {
                        manifest.properties.publisher_display_name =
                            reader.read_text(event.to_end().name())?.into_owned();
                    }
                    b"TargetDeviceFamily" => {
                        let attributes = event.attributes().flatten().collect::<Vec<_>>();
                        let platform = attributes
                            .iter()
                            .find(|attribute| attribute.key.as_ref() == b"Name")
                            .map(|platform| String::from_utf8_lossy(&platform.value))
                            .and_then(|platform| Platform::from_str(&platform).ok());
                        let min_version = attributes
                            .iter()
                            .find(|attribute| attribute.key.as_ref() == b"MinVersion")
                            .map(|min_version| String::from_utf8_lossy(&min_version.value))
                            .and_then(|min_version| MinimumOSVersion::from_str(&min_version).ok());
                        if let (Some(platform), Some(min_version)) = (platform, min_version) {
                            manifest
                                .dependencies
                                .target_device_family
                                .insert(TargetDeviceFamily {
                                    name: platform,
                                    min_version,
                                });
                        }
                    }
                    b"FileType" => {
                        if let Ok(extension) = FileExtension::from_str(
                            reader
                                .read_text(event.to_end().name())?
                                .trim_start_matches('.'),
                        ) {
                            manifest
                                .file_type_association
                                .supported_file_types
                                .insert(extension);
                        }
                    }
                    b"Capability" => {
                        let _ = event
                            .attributes()
                            .flatten()
                            .find(|attribute| attribute.key.as_ref() == b"Name")
                            .map(|attribute| String::from_utf8_lossy(&attribute.value).into_owned())
                            .is_some_and(|capability| {
                                if event
                                    .name()
                                    .prefix()
                                    .is_some_and(|prefix| prefix.as_ref() == b"rescap")
                                {
                                    manifest.capabilities.restricted.insert(capability)
                                } else {
                                    manifest.capabilities.unrestricted.insert(capability)
                                }
                            });
                    }
                    _ => continue,
                },
                Event::Eof => break,
                _ => (),
            }
        }

        let is_appx = manifest
            .dependencies
            .target_device_family
            .iter()
            .all(|target_device_family| target_device_family.min_version < MSIX_MIN_VERSION)
            && {
                appx_manifest.make_ascii_lowercase();
                !appx_manifest.contains(MSIX)
            };

        Ok(Self {
            installer: Installer {
                platform: Some(
                    manifest
                        .dependencies
                        .target_device_family
                        .iter()
                        .map(|target_device_family| target_device_family.name)
                        .collect(),
                ),
                minimum_os_version: manifest
                    .dependencies
                    .target_device_family
                    .into_iter()
                    .map(|target_device_family| target_device_family.min_version)
                    .min(),
                architecture: Architecture::from_str(&manifest.identity.processor_architecture)?,
                r#type: if is_appx {
                    Some(InstallerType::Appx)
                } else {
                    Some(InstallerType::Msix)
                },
                signature_sha_256: Some(signature_sha_256),
                upgrade_behavior: Some(UpgradeBehavior::Install),
                file_extensions: Option::from(manifest.file_type_association.supported_file_types)
                    .filter(|supported_file_types| !supported_file_types.is_empty()),
                package_family_name: Some(PackageFamilyName::new(
                    &manifest.identity.name,
                    &manifest.identity.publisher,
                )),
                capabilities: Option::from(manifest.capabilities.unrestricted)
                    .filter(|capabilities| !capabilities.is_empty()),
                restricted_capabilities: Option::from(manifest.capabilities.restricted)
                    .filter(|restricted| !restricted.is_empty()),
                apps_and_features_entries: Some(vec![AppsAndFeaturesEntry {
                    display_name: Some(manifest.properties.display_name),
                    publisher: Some(manifest.properties.publisher_display_name),
                    display_version: Some(Version::new(&manifest.identity.version)),
                    ..AppsAndFeaturesEntry::default()
                }]),
                installation_metadata: Some(InstallationMetadata {
                    default_install_location: Some(get_install_location(
                        &manifest.identity.name,
                        &manifest.identity.publisher,
                        &manifest.identity.version,
                        &manifest.identity.processor_architecture,
                        &manifest.identity.resource_id,
                    )),
                    ..InstallationMetadata::default()
                }),
                ..Installer::default()
            },
        })
    }
}

/// <https://learn.microsoft.com/uwp/schemas/appxpackage/uapmanifestschema/element-package>
#[derive(Default)]
struct Package {
    identity: Identity,
    properties: Properties,
    dependencies: Dependencies,
    capabilities: Capabilities,
    file_type_association: FileTypeAssociation,
}

/// <https://learn.microsoft.com/uwp/schemas/appxpackage/uapmanifestschema/element-identity>
#[derive(Default)]
struct Identity {
    name: String,
    processor_architecture: String,
    publisher: String,
    version: String,
    resource_id: String,
}

/// <https://learn.microsoft.com/uwp/schemas/appxpackage/uapmanifestschema/element-properties>
#[derive(Default)]
struct Properties {
    display_name: String,
    publisher_display_name: String,
}

/// <https://learn.microsoft.com/uwp/schemas/appxpackage/uapmanifestschema/element-dependencies>
#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub(super) struct Dependencies {
    pub target_device_family: BTreeSet<TargetDeviceFamily>,
}

/// <https://learn.microsoft.com/uwp/schemas/appxpackage/uapmanifestschema/element-targetdevicefamily>
#[derive(Deserialize, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct TargetDeviceFamily {
    #[serde(rename = "@Name")]
    pub name: Platform,
    #[serde(rename = "@MinVersion")]
    pub min_version: MinimumOSVersion,
}

/// <https://learn.microsoft.com/uwp/schemas/appxpackage/uapmanifestschema/element-capabilities>
#[derive(Default)]
struct Capabilities {
    restricted: BTreeSet<String>,
    unrestricted: BTreeSet<String>,
}

/// <https://learn.microsoft.com/uwp/schemas/appxpackage/uapmanifestschema/element-uap-filetypeassociation>
#[derive(Default)]
struct FileTypeAssociation {
    supported_file_types: BTreeSet<FileExtension>,
}
