mod nupkg_reader;
mod nuspec;

use std::{
    io::{self, Read, Seek, SeekFrom},
    path::Path,
};

use camino::{Utf8Path, Utf8PathBuf};
use nupkg_reader::NupkgReader;
use nuspec::NuSpec;
use quick_xml::de::from_str;
use thiserror::Error;
use tracing::debug;
use winget_types::installer::{
    AppsAndFeaturesEntry, Architecture, InstallationMetadata, Installer, InstallerSwitches,
    InstallerType, Scope,
};
use zip::ZipArchive;

use super::pe::utils::machine_from_exe_reader;
use crate::{
    analysis::{
        Installers,
        installers::pe::{PE, resource::SectionReader},
    },
    traits::FromMachine,
};

#[derive(Error, Debug)]
pub enum SquirrelError {
    #[error("File is not a Squirrel installer")]
    NotSquirrelFile,
    #[error("No nupkg found in Squirrel zip")]
    NoNupkgFound,
    #[error(transparent)]
    NuspecDeserialization(#[from] quick_xml::DeError),
    #[error(transparent)]
    Zip(#[from] zip::result::ZipError),
    #[error(transparent)]
    Io(#[from] io::Error),
}

pub struct Squirrel {
    pub architecture: Architecture,
    pub nuspec: NuSpec,
    pub is_velopack: bool,
}

impl Squirrel {
    pub fn new<R: Read + Seek>(mut reader: R, pe: &PE) -> Result<Self, SquirrelError> {
        let mut is_velopack = false;

        let mut nupkg = if let Ok(Ok(mut zip)) = pe
            .find_resource_by_name(&mut reader, "DATA")
            .map(ZipArchive::new)
        {
            // Squirrel
            let nupkg_name = zip
                .file_names()
                .find(|name| {
                    Path::new(name)
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("nupkg"))
                })
                .ok_or(SquirrelError::NoNupkgFound)?
                .to_owned();
            let mut nupkg_reader = zip.by_name(&nupkg_name)?;

            // Copy the .nupkg file to a temporary file
            let mut nupkg_file = tempfile::tempfile()?;
            io::copy(&mut nupkg_reader, &mut nupkg_file)?;
            nupkg_file.seek(SeekFrom::Start(0))?;

            ZipArchive::new(NupkgReader::File(nupkg_file))
                .map_err(|_| SquirrelError::NotSquirrelFile)?
        } else {
            // Velopack (Squirrel fork)
            let header_offset = pe.overlay_offset().ok_or(SquirrelError::NotSquirrelFile)?;
            let section_reader = SectionReader::from_offset(reader, header_offset)?;
            is_velopack = true;

            ZipArchive::new(NupkgReader::Section(section_reader))
                .map_err(|_| SquirrelError::NotSquirrelFile)?
        };

        let nuspec_filename = nupkg
            .file_names()
            .find(|name| name.ends_with(".nuspec"))
            .ok_or(SquirrelError::NotSquirrelFile)?
            .to_owned();

        debug!(is_velopack);

        let nuspec_data = io::read_to_string(nupkg.by_name(&nuspec_filename)?)?;
        debug!(%nuspec_data);
        let nuspec: NuSpec = from_str(&nuspec_data)?;

        let entrypoint = nupkg
            .file_names()
            .map(Utf8Path::new)
            .filter(|name| {
                name.extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("exe"))
            })
            .find(|name| {
                name.file_stem().is_some_and(|stem| {
                    nuspec
                        .main_exe()
                        .is_some_and(|main_exe| main_exe.eq_ignore_ascii_case(stem))
                        || stem.eq_ignore_ascii_case(nuspec.id())
                        || nuspec
                            .title()
                            .is_some_and(|title| title.eq_ignore_ascii_case(stem))
                })
            })
            .map(Utf8Path::to_path_buf);

        let architecture = entrypoint
            .and_then(|entrypoint| {
                let reader = nupkg.by_name(entrypoint.as_str()).ok()?;

                let machine = machine_from_exe_reader(reader).ok()?;
                Some(Architecture::from_machine(machine))
            })
            .unwrap_or_else(|| Architecture::from_machine(pe.machine()));

        Ok(Self {
            architecture,
            nuspec,
            is_velopack,
        })
    }
}

impl Installers for Squirrel {
    fn installers(&self) -> Vec<Installer> {
        let nuspec = &self.nuspec;

        let switches = if self.is_velopack {
            InstallerSwitches::builder()
                .silent("--silent".parse().unwrap())
                .silent_with_progress("--silent".parse().unwrap())
                .install_location(r#"--installto "<INSTALLPATH>""#.parse().unwrap())
                .log(r#"--log "<LOGPATH>""#.parse().unwrap())
                .build()
        } else {
            InstallerSwitches::builder()
                .silent("--silent".parse().unwrap())
                .silent_with_progress("--silent".parse().unwrap())
                .build()
        };

        vec![Installer {
            architecture: self.architecture,
            r#type: Some(InstallerType::Exe),
            scope: Some(Scope::User),
            product_code: Some(nuspec.id().to_owned()),
            apps_and_features_entries: AppsAndFeaturesEntry::builder()
                .display_name(nuspec.title().unwrap_or_else(|| nuspec.id()).to_owned())
                .publisher(nuspec.authors().to_owned())
                .display_version(nuspec.version().to_owned())
                .product_code(nuspec.id().to_owned())
                .build()
                .into(),
            switches,
            installation_metadata: InstallationMetadata {
                default_install_location: Some(Utf8PathBuf::from(format!(
                    r"%LocalAppData%\{}",
                    nuspec.id()
                ))),
                ..InstallationMetadata::default()
            },
            ..Installer::default()
        }]
    }
}
