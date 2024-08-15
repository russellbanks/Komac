use crate::file_analyser::FileAnalyser;
use crate::manifest::print_manifest;
use crate::manifests::installer_manifest::{AppsAndFeaturesEntry, InstallationMetadata, Installer};
use crate::types::sha_256::Sha256String;
use anstream::stdout;
use camino::{Utf8Path, Utf8PathBuf};
use clap::Parser;
use color_eyre::eyre::bail;
use color_eyre::Result;
use memmap2::Mmap;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs::File;
use std::mem;

#[derive(Parser)]
pub struct Analyse {
    #[arg(value_parser = is_valid_file, value_hint = clap::ValueHint::FilePath)]
    file_path: Utf8PathBuf,
}

impl Analyse {
    pub fn run(self) -> Result<()> {
        let file = File::open(&self.file_path)?;
        let mmap = unsafe { Mmap::map(&file) }?;
        let file_name = self
            .file_path
            .file_name()
            .unwrap_or_else(|| self.file_path.as_str());
        let mut analyser = FileAnalyser::new(&mmap, file_name)?;
        let mut installer = Installer {
            installer_locale: analyser.product_language,
            platform: analyser.platform,
            minimum_os_version: analyser.minimum_os_version,
            architecture: analyser.architecture.unwrap(),
            installer_type: Some(analyser.installer_type),
            nested_installer_type: analyser
                .zip
                .as_mut()
                .and_then(|zip| mem::take(&mut zip.nested_installer_type)),
            nested_installer_files: analyser
                .zip
                .as_mut()
                .and_then(|zip| mem::take(&mut zip.nested_installer_files)),
            scope: analyser.scope,
            installer_sha_256: Sha256String::from_hasher(&Sha256::digest(&mmap))?,
            signature_sha_256: analyser.signature_sha_256,
            file_extensions: analyser.file_extensions,
            package_family_name: analyser.package_family_name,
            product_code: analyser.product_code.clone(),
            capabilities: analyser.capabilities,
            restricted_capabilities: analyser.restricted_capabilities,
            elevation_requirement: analyser.elevation_requirement,
            ..Installer::default()
        };
        if analyser.display_name.is_some()
            || analyser.display_publisher.is_some()
            || analyser.display_version.is_some()
            || analyser.upgrade_code.is_some()
        {
            installer.apps_and_features_entries = Some(BTreeSet::from([AppsAndFeaturesEntry {
                display_name: analyser.display_name,
                publisher: analyser.display_publisher,
                display_version: analyser.display_version,
                product_code: analyser.product_code,
                upgrade_code: analyser.upgrade_code,
                ..AppsAndFeaturesEntry::default()
            }]));
        }
        if let Some(install_location) = analyser.default_install_location {
            installer.installation_metadata = Some(InstallationMetadata {
                default_install_location: Some(install_location),
                ..InstallationMetadata::default()
            });
        }
        let mut lock = stdout().lock();
        print_manifest(&mut lock, &serde_yaml::to_string(&installer)?);
        Ok(())
    }
}

fn is_valid_file(path: &str) -> Result<Utf8PathBuf> {
    let path = Utf8Path::new(path);
    if !path.exists() {
        bail!("{path} does not exist")
    }
    if !path.is_file() {
        bail!("{path} is not a file")
    }
    Ok(path.to_path_buf())
}
