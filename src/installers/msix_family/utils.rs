use std::io::{Read, Seek};

use camino::Utf8PathBuf;
use color_eyre::eyre::Result;
use package_family_name::PackageFamilyName;
use zip::ZipArchive;

use crate::installers::msi::RELATIVE_PROGRAM_FILES_64;
use crate::installers::msix_family::APPX_SIGNATURE_P7X;
use crate::types::sha_256::Sha256String;

pub fn read_manifest<R: Read + Seek>(zip: &mut ZipArchive<R>, path: &str) -> Result<String> {
    let mut appx_manifest_file = zip.by_name(path)?;
    let mut appx_manifest = String::with_capacity(usize::try_from(appx_manifest_file.size())?);
    appx_manifest_file.read_to_string(&mut appx_manifest)?;
    Ok(appx_manifest)
}

pub fn hash_signature<R: Read + Seek>(zip: &mut ZipArchive<R>) -> Result<Sha256String> {
    let signature_file = zip.by_name(APPX_SIGNATURE_P7X)?;
    Sha256String::from_reader(signature_file)
}

pub fn get_install_location(
    name: &str,
    publisher: &str,
    version: &str,
    architecture: &str,
    resource_id: &str,
) -> Utf8PathBuf {
    const WINDOWS_APPS: &str = "WindowsApps";

    let mut path = Utf8PathBuf::from(RELATIVE_PROGRAM_FILES_64);
    path.push(WINDOWS_APPS);
    path.push(format!(
        "{name}_{version}_{architecture}_{resource_id}_{}",
        PackageFamilyName::get_id(publisher)
    ));
    path
}
