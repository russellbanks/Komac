use std::{
    collections::{BTreeSet, HashMap},
    io,
    io::{Read, Seek, SeekFrom},
    mem,
    path::Path,
};

use camino::{Utf8Path, Utf8PathBuf};
use color_eyre::eyre::Result;
use inquire::{CustomType, MultiSelect, min_length};
use tracing::debug;
use winget_types::installer::{
    Installer, InstallerType, NestedInstallerFiles, PortableCommandAlias,
};
use zip::ZipArchive;

use super::super::Analyzer;
use crate::{i18n::fl, prompts::handle_inquire_error};

const VALID_NESTED_FILE_EXTENSIONS: [&str; 6] =
    ["msix", "msi", "appx", "exe", "msixbundle", "appxbundle"];

const IGNORABLE_FOLDERS: [&str; 2] = ["__MACOSX", "resources"];

pub struct Zip<R: Read + Seek> {
    archive: ZipArchive<R>,
    pub possible_installer_files: Vec<Utf8PathBuf>,
    pub installers: Vec<Installer>,
}

impl<R: Read + Seek> Zip<R> {
    pub fn new(reader: R) -> Result<Self> {
        let mut zip = ZipArchive::new(reader)?;

        let possible_installer_files = zip
            .file_names()
            .flatten()
            .filter(|file_name| {
                VALID_NESTED_FILE_EXTENSIONS.iter().any(|file_extension| {
                    Path::new(&**file_name)
                        .extension()
                        .is_some_and(|extension| extension.eq_ignore_ascii_case(file_extension))
                })
            })
            .filter(|file_name| {
                // Ignore folders that the main executable is unlikely to be in
                Utf8Path::new(file_name).components().all(|component| {
                    IGNORABLE_FOLDERS
                        .iter()
                        .all(|folder| !component.as_str().eq_ignore_ascii_case(folder))
                })
            })
            .map(|name| Utf8PathBuf::from(&name))
            .collect::<Vec<_>>();

        debug!(?possible_installer_files);

        let installer_type_counts = VALID_NESTED_FILE_EXTENSIONS
            .iter()
            .map(|file_extension| {
                (
                    file_extension,
                    possible_installer_files
                        .iter()
                        .filter(|file_name| {
                            file_name.extension().is_some_and(|extension| {
                                extension.eq_ignore_ascii_case(file_extension)
                            })
                        })
                        .count(),
                )
            })
            .collect::<HashMap<_, _>>();

        let mut nested_installer_files = BTreeSet::new();
        let mut installers = None;

        // If there's only one valid file in the zip, extract and analyze it
        if installer_type_counts
            .values()
            .filter(|&&count| count == 1)
            .count()
            == 1
        {
            nested_installer_files = BTreeSet::from([NestedInstallerFiles {
                relative_file_path: possible_installer_files[0].clone(),
                portable_command_alias: None,
            }]);
            installers = analyze_nested_files(&mut zip, &nested_installer_files)?;
        }

        Ok(Self {
            archive: zip,
            possible_installer_files,
            installers: installers.unwrap_or_else(|| {
                vec![Installer {
                    r#type: Some(InstallerType::Zip),
                    nested_installer_files,
                    ..Installer::default()
                }]
            }),
        })
    }

    pub fn analyze_nested_files(
        &mut self,
        nested_installer_files: &BTreeSet<NestedInstallerFiles>,
    ) -> Result<Option<Vec<Installer>>> {
        analyze_nested_files(&mut self.archive, nested_installer_files)
    }

    pub fn prompt(&mut self) -> Result<()> {
        if !&self.possible_installer_files.is_empty() {
            let chosen = MultiSelect::new(
                &fl!("prompt-nested-files"),
                mem::take(&mut self.possible_installer_files),
            )
            .with_validator(min_length!(1))
            .prompt()
            .map_err(handle_inquire_error)?;
            let first_choice = chosen.first().unwrap();
            let mut temp_file = tempfile::tempfile()?;
            io::copy(
                &mut self.archive.by_name(first_choice.as_str())?,
                &mut temp_file,
            )?;
            temp_file.seek(SeekFrom::Start(0))?;
            let file_analyzer = Analyzer::new(
                &mut temp_file,
                first_choice.file_name().unwrap_or(first_choice.as_str()),
            )?;
            let nested_installer_files = chosen
                .into_iter()
                .map(|path| {
                    Ok(NestedInstallerFiles {
                        portable_command_alias: if file_analyzer.installers.first().is_some_and(
                            |installer| installer.r#type == Some(InstallerType::Portable),
                        ) {
                            CustomType::<PortableCommandAlias>::new(&fl!(
                                "prompt-nested-portable-command-alias",
                                path = path.as_str()
                            ))
                            .prompt_skippable()
                            .map_err(handle_inquire_error)?
                        } else {
                            None
                        },
                        relative_file_path: path,
                    })
                })
                .collect::<Result<BTreeSet<_>>>()?;
            self.installers = file_analyzer
                .installers
                .into_iter()
                .map(|installer| Installer {
                    nested_installer_type: installer
                        .r#type
                        .and_then(|installer_type| installer_type.try_into().ok()),
                    nested_installer_files: nested_installer_files.clone(),
                    ..installer
                })
                .collect();
        }
        Ok(())
    }
}

fn analyze_nested_files<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    nested_installer_files: &BTreeSet<NestedInstallerFiles>,
) -> Result<Option<Vec<Installer>>> {
    let Some(first_file) = nested_installer_files.first() else {
        return Ok(None);
    };
    let Ok(mut nested_file) = archive.by_name(first_file.relative_file_path.as_str()) else {
        return Ok(None);
    };
    let mut temp_file = tempfile::tempfile()?;
    io::copy(&mut nested_file, &mut temp_file)?;
    temp_file.seek(SeekFrom::Start(0))?;
    let file_analyzer = Analyzer::new(&mut temp_file, first_file.relative_file_path.as_str())?;
    Ok(Some(
        file_analyzer
            .installers
            .into_iter()
            .map(|installer| Installer {
                r#type: Some(InstallerType::Zip),
                nested_installer_type: installer
                    .r#type
                    .and_then(|installer_type| installer_type.try_into().ok()),
                nested_installer_files: nested_installer_files.clone(),
                ..installer
            })
            .collect(),
    ))
}
