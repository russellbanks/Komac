use std::{
    io,
    io::{Read, Seek},
};

use camino::Utf8PathBuf;
use color_eyre::eyre::Result;
use winget_types::{Sha256String, package_family_name::PublisherId};
use zip::ZipArchive;

use crate::installers::{msix_family::APPX_SIGNATURE_P7X, utils::RELATIVE_PROGRAM_FILES_64};

pub fn read_manifest<R: Read + Seek>(zip: &mut ZipArchive<R>, path: &str) -> Result<String> {
    let mut appx_manifest_file = zip.by_name(path)?;
    let mut appx_manifest = String::with_capacity(usize::try_from(appx_manifest_file.size())?);
    appx_manifest_file.read_to_string(&mut appx_manifest)?;
    Ok(appx_manifest)
}

pub fn hash_signature<R: Read + Seek>(zip: &mut ZipArchive<R>) -> io::Result<Sha256String> {
    let signature_file = zip.by_name(APPX_SIGNATURE_P7X)?;
    Sha256String::hash_from_reader(signature_file)
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
        PublisherId::new(publisher)
    ));
    path
}
