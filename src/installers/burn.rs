use crate::installers::msi::Msi;
use crate::installers::traits::InstallSpec;
use crate::manifests::installer_manifest::Scope;
use crate::types::architecture::Architecture;
use crate::types::installer_type::InstallerType;
use crate::types::language_tag::LanguageTag;
use crate::types::version::Version;
use camino::Utf8PathBuf;
use std::io::Cursor;
use std::io::{Error, ErrorKind, Result};
use yara_x::mods::pe::{Resource, ResourceType};
use yara_x::mods::PE;

pub struct Burn(Option<Msi>);

impl Burn {
    pub fn new(data: &[u8], pe: &PE) -> Result<Self> {
        Self::get_msi_resource(pe).map_or_else(
            || {
                if Self::has_burn_header(pe) {
                    Ok(Self(None))
                } else {
                    Err(Error::new(
                        ErrorKind::NotFound,
                        "Could not find Msi resource",
                    ))
                }
            },
            |msi_resource| Self::extract_msi(data, msi_resource).map(|msi| Self(Some(msi))),
        )
    }

    fn has_burn_header(pe: &PE) -> bool {
        const WIXBURN_HEADER: &[u8] = b".wixburn";

        pe.sections
            .iter()
            .any(|section| section.name() == WIXBURN_HEADER)
    }

    fn extract_msi(data: &[u8], msi_resource: &Resource) -> Result<Msi> {
        let offset = msi_resource.offset() as usize;
        let data = &data[offset..offset + msi_resource.length() as usize];
        Msi::new(Cursor::new(data))
    }

    fn get_msi_resource(pe: &PE) -> Option<&Resource> {
        const MSI: &[u8] = b"M\0S\0I\0";

        pe.resources
            .iter()
            .filter(|resource| resource.type_() == ResourceType::RESOURCE_TYPE_RCDATA)
            .find(|resource| resource.name_string() == MSI)
    }
}

impl InstallSpec for Burn {
    fn r#type(&self) -> InstallerType {
        InstallerType::Burn
    }

    fn architecture(&self) -> Option<Architecture> {
        self.0.as_ref().map(|msi| msi.architecture)
    }

    fn display_name(&self) -> Option<String> {
        self.0.as_ref().and_then(|msi| msi.product_name.clone())
    }

    fn display_publisher(&self) -> Option<String> {
        self.0.as_ref().and_then(|msi| msi.manufacturer.clone())
    }

    fn display_version(&self) -> Option<Version> {
        self.0
            .as_ref()
            .and_then(|msi| msi.product_version.as_deref().map(Version::new))
    }

    fn product_code(&self) -> Option<String> {
        self.0.as_ref().and_then(|msi| msi.product_code.clone())
    }

    fn locale(&self) -> Option<LanguageTag> {
        self.0.as_ref().and_then(|msi| msi.product_language.clone())
    }

    fn scope(&self) -> Option<Scope> {
        self.0.as_ref().and_then(|msi| msi.all_users)
    }

    fn install_location(&self) -> Option<Utf8PathBuf> {
        self.0.as_ref().and_then(|msi| msi.install_location.clone())
    }

    fn upgrade_code(&self) -> Option<String> {
        self.0.as_ref().and_then(|msi| msi.upgrade_code.clone())
    }
}
