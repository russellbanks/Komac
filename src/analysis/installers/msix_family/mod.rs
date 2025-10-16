pub mod bundle;
mod utils;

use std::{
    collections::BTreeSet,
    io::{Read, Seek},
};

use color_eyre::eyre::Result;
use quick_xml::{Reader, events::Event};
use winget_types::{
    Sha256String,
    installer::{
        AppsAndFeaturesEntry, Architecture, Capability, FileExtension, InstallationMetadata,
        Installer, InstallerType, MinimumOSVersion, PackageFamilyName, Platform,
        RestrictedCapability, UpgradeBehavior,
    },
};
use zip::ZipArchive;

use super::msix_family::utils::{get_install_location, hash_signature, read_manifest};
use crate::{
    analysis::{Installers, extensions::MSIX},
    traits::AsciiExt,
};

pub struct Msix {
    appx_manifest: String,
    pub signature_sha_256: Sha256String,
    pub manifest: Package,
}

const APPX_MANIFEST_XML: &str = "AppxManifest.xml";
pub const APPX_SIGNATURE_P7X: &str = "AppxSignature.p7x";

const MSIX_MIN_VERSION: MinimumOSVersion = MinimumOSVersion::new(10, 0, 17763, 0);

impl Msix {
    pub fn new<R: Read + Seek>(reader: R) -> Result<Self> {
        let mut zip = ZipArchive::new(reader)?;

        let appx_manifest = read_manifest(&mut zip, APPX_MANIFEST_XML)?;

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
                                _ => {}
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
                    _ => {}
                },
                Event::Eof => break,
                _ => {}
            }
        }

        Ok(Self {
            appx_manifest,
            signature_sha_256,
            manifest,
        })
    }
}

impl Installers for Msix {
    fn installers(&self) -> Vec<Installer> {
        let is_appx = self
            .manifest
            .dependencies
            .target_device_family
            .iter()
            .all(|target_device_family| target_device_family.min_version < MSIX_MIN_VERSION)
            && !self.appx_manifest.contains_ignore_ascii_case(MSIX);

        vec![Installer {
            platform: self
                .manifest
                .dependencies
                .target_device_family
                .iter()
                .map(|target_device_family| target_device_family.name)
                .collect(),
            minimum_os_version: self
                .manifest
                .dependencies
                .target_device_family
                .iter()
                .map(|target_device_family| target_device_family.min_version)
                .min(),
            architecture: self
                .manifest
                .identity
                .processor_architecture
                .parse()
                .unwrap_or(Architecture::X86),
            r#type: if is_appx {
                Some(InstallerType::Appx)
            } else {
                Some(InstallerType::Msix)
            },
            signature_sha_256: Some(self.signature_sha_256.clone()),
            upgrade_behavior: Some(UpgradeBehavior::Install),
            file_extensions: self
                .manifest
                .file_type_association
                .supported_file_types
                .clone(),
            package_family_name: Some(PackageFamilyName::new(
                self.manifest.identity.name.clone(),
                &self.manifest.identity.publisher,
            )),
            capabilities: self.manifest.capabilities.unrestricted.clone(),
            restricted_capabilities: self.manifest.capabilities.restricted.clone(),
            apps_and_features_entries: AppsAndFeaturesEntry::builder()
                .display_name(&self.manifest.properties.display_name)
                .publisher(&self.manifest.properties.publisher_display_name)
                .display_version(&self.manifest.identity.version)
                .build()
                .into(),
            installation_metadata: InstallationMetadata {
                default_install_location: Some(get_install_location(
                    &self.manifest.identity.name,
                    &self.manifest.identity.publisher,
                    &self.manifest.identity.version,
                    &self.manifest.identity.processor_architecture,
                    &self.manifest.identity.resource_id,
                )),
                ..InstallationMetadata::default()
            },
            ..Installer::default()
        }]
    }
}

/// <https://learn.microsoft.com/uwp/schemas/appxpackage/uapmanifestschema/element-package>
#[derive(Clone, Default)]
pub struct Package {
    identity: Identity,
    properties: Properties,
    dependencies: Dependencies,
    capabilities: Capabilities,
    file_type_association: FileTypeAssociation,
}

/// <https://learn.microsoft.com/uwp/schemas/appxpackage/uapmanifestschema/element-identity>
#[derive(Clone, Default)]
pub struct Identity {
    name: String,
    processor_architecture: String,
    publisher: String,
    version: String,
    resource_id: String,
}

/// <https://learn.microsoft.com/uwp/schemas/appxpackage/uapmanifestschema/element-properties>
#[derive(Clone, Default)]
pub struct Properties {
    display_name: String,
    publisher_display_name: String,
}

/// <https://learn.microsoft.com/uwp/schemas/appxpackage/uapmanifestschema/element-dependencies>
#[derive(Clone, Default)]
pub struct Dependencies {
    pub target_device_family: BTreeSet<TargetDeviceFamily>,
}

/// <https://learn.microsoft.com/uwp/schemas/appxpackage/uapmanifestschema/element-targetdevicefamily>
#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct TargetDeviceFamily {
    pub name: Platform,
    pub min_version: MinimumOSVersion,
}

/// <https://learn.microsoft.com/uwp/schemas/appxpackage/uapmanifestschema/element-capabilities>
#[derive(Clone, Default)]
pub struct Capabilities {
    restricted: BTreeSet<RestrictedCapability>,
    unrestricted: BTreeSet<Capability>,
}

/// <https://learn.microsoft.com/uwp/schemas/appxpackage/uapmanifestschema/element-uap-filetypeassociation>
#[derive(Clone, Default)]
pub struct FileTypeAssociation {
    supported_file_types: BTreeSet<FileExtension>,
}
