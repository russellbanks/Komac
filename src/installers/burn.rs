use crate::installers::msi::Msi;
use crate::manifests::installer_manifest::Installer;
use crate::types::installer_type::InstallerType;
use std::io::Cursor;
use std::io::{Error, ErrorKind, Result};
use yara_x::mods::pe::{Resource, ResourceType};
use yara_x::mods::PE;

pub struct Burn {
    pub installer: Installer,
}

impl Burn {
    pub fn new(data: &[u8], pe: &PE) -> Result<Self> {
        let msi = Self::get_msi_resource(pe).map_or_else(
            || {
                if Self::has_burn_header(pe) {
                    Ok(None)
                } else {
                    Err(Error::new(
                        ErrorKind::NotFound,
                        "Could not find Msi resource",
                    ))
                }
            },
            |msi_resource| Self::extract_msi(data, msi_resource).map(Some),
        )?;
        Ok(Self {
            installer: Installer {
                r#type: Some(InstallerType::Burn),
                ..msi.map(|msi| msi.installer).unwrap_or_default()
            },
        })
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
