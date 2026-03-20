mod nupkg_reader;
mod nuspec;

use std::{
    fs::File,
    io::{self, Cursor, Read, Seek, SeekFrom},
};

use camino::Utf8Path;
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
    #[error("No nuspec found in nupkg")]
    NoNuspecFound,
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
        // TODO: Get first section
        let data_reader = pe.data(&mut reader)?;

        let mut is_velopack = false;

        let mut nupkg = match ZipArchive::new(data_reader) {
            // Squirrel
            Ok(mut zip) => {
                let nupkg_name = zip
                    .file_names()
                    .find(|name| name.ends_with(".nupkg"))
                    .ok_or(SquirrelError::NoNupkgFound)?
                    .to_owned();
                let mut nupkg_reader = zip.by_name(&nupkg_name)?;

                // Copy the .nupkg file to a temporary file
                let mut nupkg_file = tempfile::tempfile()?;
                io::copy(&mut nupkg_reader, &mut nupkg_file)?;
                nupkg_file.seek(SeekFrom::Start(0))?;

                ZipArchive::new(NupkgReader::File(nupkg_file))
                    .map_err(|_| SquirrelError::NotSquirrelFile)?
            }
            // Velopack (Squirrel fork)
            Err(_) => {
                let header_offset = pe.overlay_offset().ok_or(SquirrelError::NotSquirrelFile)?;
                let section_reader = SectionReader::from_offset(reader, header_offset)?;
                is_velopack = true;

                ZipArchive::new(NupkgReader::Memory(section_reader))
                    .map_err(|_| SquirrelError::NotSquirrelFile)?
            }
        };

        debug!(is_velopack);

        let nuspec_filename = nupkg
            .file_names()
            .find(|name| name.ends_with(".nuspec"))
            .ok_or(SquirrelError::NoNuspecFound)?
            .to_owned();
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
                        .metadata
                        .main_exe
                        .as_deref()
                        .is_some_and(|main_exe| main_exe.eq_ignore_ascii_case(stem))
                        || stem.eq_ignore_ascii_case(&nuspec.metadata.id)
                        || nuspec
                            .title()
                            .is_some_and(|title| stem.eq_ignore_ascii_case(title))
                })
            })
            .map(Utf8Path::to_path_buf);
        let architecture = entrypoint
            .and_then(|name| {
                let mut exe_data = Vec::new();
                nupkg
                    .by_name(name.as_str())
                    .ok()?
                    .read_to_end(&mut exe_data)
                    .ok()?;
                /*yara_x::mods::invoke::<PE>(&exe_data)
                .map(|pe| Architecture::from_machine(pe.machine()))*/
                None
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
        let metadata = &self.nuspec.metadata;

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
            product_code: Some(metadata.id.clone()),
            apps_and_features_entries: AppsAndFeaturesEntry::builder()
                .display_name(
                    metadata
                        .title
                        .clone()
                        .unwrap_or_else(|| metadata.id.clone()),
                )
                .publisher(metadata.authors.clone())
                .display_version(metadata.version.clone())
                .product_code(metadata.id.clone())
                .build()
                .into(),
            switches,
            installation_metadata: InstallationMetadata {
                default_install_location: Some(camino::Utf8PathBuf::from(format!(
                    "%LocalAppData%\\{}",
                    metadata.id
                ))),
                ..InstallationMetadata::default()
            },
            ..Installer::default()
        }]
    }
}
