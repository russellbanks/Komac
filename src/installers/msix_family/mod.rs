pub mod bundle;
mod utils;

use std::{
    collections::BTreeSet,
    io::{Read, Seek},
    str::FromStr,
};

use color_eyre::eyre::Result;
use quick_xml::{Reader, events::Event};
use winget_types::installer::{
    AppsAndFeaturesEntry, Architecture, Capability, FileExtension, InstallationMetadata, Installer,
    InstallerType, MinimumOSVersion, PackageFamilyName, Platform, RestrictedCapability,
    UpgradeBehavior,
};
use zip::ZipArchive;

use crate::{
    file_analyser::MSIX,
    installers::msix_family::utils::{get_install_location, hash_signature, read_manifest},
};

pub struct Msix {
    pub installer: Installer,
}

const APPX_MANIFEST_XML: &str = "AppxManifest.xml";
pub const APPX_SIGNATURE_P7X: &str = "AppxSignature.p7x";

const MSIX_MIN_VERSION: MinimumOSVersion = MinimumOSVersion::new(10, 0, 17763, 0);

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
                                    html_escape::decode_html_entities_to_string(
                                        String::from_utf8_lossy(&attribute.value),
                                        &mut manifest.identity.publisher,
                                    );
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
                        let mut name = None;
                        let mut min_version = None;
                        for attribute in event.attributes().flatten() {
                            if attribute.key.as_ref() == b"Name"
                                && let Ok(platform) = std::str::from_utf8(&attribute.value)
                                && let Ok(platform) = platform.parse()
                            {
                                name = Some(platform);
                            } else if attribute.key.as_ref() == b"MinVersion"
                                && let Ok(version) = std::str::from_utf8(&attribute.value)
                                && let Ok(version) = version.parse()
                            {
                                min_version = Some(version);
                            }
                        }
                        if let (Some(name), Some(min_version)) = (name, min_version) {
                            manifest
                                .dependencies
                                .target_device_family
                                .insert(TargetDeviceFamily { name, min_version });
                        }
                    }
                    b"FileType" => {
                        if let Ok(extension) = FileExtension::new(
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
                        if let Some(attribute) = event
                            .attributes()
                            .flatten()
                            .find(|attribute| attribute.key.as_ref() == b"Name")
                            && let Ok(capability) = std::str::from_utf8(&attribute.value)
                        {
                            if event
                                .name()
                                .prefix()
                                .is_some_and(|prefix| prefix.into_inner() == b"rescap")
                            {
                                if let Ok(restricted_capability) =
                                    capability.parse::<RestrictedCapability>()
                                {
                                    manifest
                                        .capabilities
                                        .restricted
                                        .insert(restricted_capability);
                                }
                            } else if let Ok(capability) = capability.parse::<Capability>() {
                                manifest.capabilities.unrestricted.insert(capability);
                            }
                        }
                    }
                    _ => continue,
                },
                Event::Eof => break,
                _ => {}
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
                platform: manifest
                    .dependencies
                    .target_device_family
                    .iter()
                    .map(|target_device_family| target_device_family.name)
                    .collect(),
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
                file_extensions: manifest.file_type_association.supported_file_types,
                package_family_name: Some(PackageFamilyName::new(
                    &manifest.identity.name,
                    &manifest.identity.publisher,
                )),
                capabilities: manifest.capabilities.unrestricted,
                restricted_capabilities: manifest.capabilities.restricted,
                apps_and_features_entries: vec![
                    AppsAndFeaturesEntry::new()
                        .with_display_name(manifest.properties.display_name)
                        .with_publisher(manifest.properties.publisher_display_name)
                        .with_display_version(&manifest.identity.version),
                ],
                installation_metadata: InstallationMetadata {
                    default_install_location: Some(get_install_location(
                        &manifest.identity.name,
                        &manifest.identity.publisher,
                        &manifest.identity.version,
                        &manifest.identity.processor_architecture,
                        &manifest.identity.resource_id,
                    )),
                    ..InstallationMetadata::default()
                },
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
#[derive(Default)]
pub struct Dependencies {
    pub target_device_family: BTreeSet<TargetDeviceFamily>,
}

/// <https://learn.microsoft.com/uwp/schemas/appxpackage/uapmanifestschema/element-targetdevicefamily>
#[derive(Eq, Ord, PartialEq, PartialOrd)]
pub struct TargetDeviceFamily {
    pub name: Platform,
    pub min_version: MinimumOSVersion,
}

/// <https://learn.microsoft.com/uwp/schemas/appxpackage/uapmanifestschema/element-capabilities>
#[derive(Default)]
struct Capabilities {
    restricted: BTreeSet<RestrictedCapability>,
    unrestricted: BTreeSet<Capability>,
}

/// <https://learn.microsoft.com/uwp/schemas/appxpackage/uapmanifestschema/element-uap-filetypeassociation>
#[derive(Default)]
struct FileTypeAssociation {
    supported_file_types: BTreeSet<FileExtension>,
}
