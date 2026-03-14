mod nuspec;

use std::io::{self, Cursor, Read, Seek, SeekFrom};

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
    analysis::{Installers, installers::pe::PE},
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
        pe.data(&mut reader)?;

        panic!();

        /*let resource = pe.resources.first().ok_or(SquirrelError::NotSquirrelFile)?;
        reader.seek(SeekFrom::Start(resource.offset().into()))?;
        let mut resource_data = vec![0; resource.length() as usize];
        reader.read_exact(&mut resource_data)?;

        let mut is_velopack = false;

        let nupkg = match ZipArchive::new(Cursor::new(&resource_data)) {
            // Squirrel
            Ok(mut zip) => {
                let nupkg_name = zip
                    .file_names()
                    .find(|name| name.ends_with(".nupkg"))
                    .map(String::from)
                    .ok_or(SquirrelError::NoNupkgFound)?;
                let mut nupkg_data = Vec::new();
                zip.by_name(&nupkg_name)?.read_to_end(&mut nupkg_data)?;
                ZipArchive::new(Cursor::new(nupkg_data)).or(Err(SquirrelError::NotSquirrelFile))?
            }
            // Velopack (Squirrel fork)
            Err(_) => {
                let header_offset = pe.overlay_offset().ok_or(SquirrelError::NotSquirrelFile)?;
                reader.seek(SeekFrom::Start(header_offset))?;
                let mut overlay_data = Vec::new();
                reader.read_to_end(&mut overlay_data)?;
                is_velopack = true;
                ZipArchive::new(Cursor::new(overlay_data))
                    .or(Err(SquirrelError::NotSquirrelFile))?
            }
        };

        let nuspec_name = nupkg
            .file_names()
            .find(|name| name.ends_with(".nuspec"))
            .map(String::from)
            .ok_or(SquirrelError::NoNuspecFound)?;
        let nuspec_data = io::read_to_string(nupkg.by_name(&nuspec_name)?)?;
        debug!(%nuspec_data);
        let nuspec: NuSpec = from_str(&nuspec_data)?;

        let entrypoint = nupkg
            .file_names()
            .filter(|name| name.ends_with(".exe"))
            .find(|name| {
                name.rsplit('/')
                    .next()
                    .and_then(|f| f.strip_suffix(".exe"))
                    .is_some_and(|stem| {
                        nuspec
                            .metadata
                            .main_exe
                            .as_ref()
                            .is_some_and(|main_exe| stem.eq_ignore_ascii_case(main_exe))
                            || stem.eq_ignore_ascii_case(&nuspec.metadata.id)
                            || nuspec
                                .metadata
                                .title
                                .as_ref()
                                .is_some_and(|title| stem.eq_ignore_ascii_case(title))
                    })
            })
            .map(String::from);
        let architecture = entrypoint
            .and_then(|name| {
                let mut exe_data = Vec::new();
                nupkg.by_name(&name).ok()?.read_to_end(&mut exe_data).ok()?;
                /*yara_x::mods::invoke::<PE>(&exe_data)
                .map(|pe| Architecture::from_machine(pe.machine()))*/
                None
            })
            .unwrap_or_else(|| Architecture::from_machine(pe.machine()));

        Ok(Self {
            architecture,
            nuspec,
            is_velopack,
        })*/
    }
}

impl Installers for Squirrel {
    fn installers(&self) -> Vec<Installer> {
        let metadata = &self.nuspec.metadata;

        let switches = if self.is_velopack {
            InstallerSwitches::builder()
                .silent("--silent".parse().unwrap())
                .silent_with_progress("--silent".parse().unwrap())
                .install_location("--installto \"<INSTALLPATH>\"".parse().unwrap())
                .log("--log \"<LOGPATH>\"".parse().unwrap())
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
