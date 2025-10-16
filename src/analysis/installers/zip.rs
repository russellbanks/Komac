use std::{
    collections::{BTreeSet, HashMap},
    io,
    io::{Read, Seek},
    mem,
};

use camino::{Utf8Path, Utf8PathBuf};
use color_eyre::eyre::Result;
use inquire::{MultiSelect, min_length};
use memmap2::Mmap;
use tracing::debug;
use winget_types::installer::{Installer, InstallerType, NestedInstallerFiles};
use zip::ZipArchive;

use super::super::Analyzer;
use crate::prompts::{handle_inquire_error, text::required_prompt};

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
            .map(Utf8Path::new)
            .filter(|file_name| {
                VALID_NESTED_FILE_EXTENSIONS.iter().any(|file_extension| {
                    file_name
                        .extension()
                        .is_some_and(|extension| extension.eq_ignore_ascii_case(file_extension))
                })
            })
            .filter(|file_name| {
                // Ignore folders that the main executable is unlikely to be in
                file_name.components().all(|component| {
                    IGNORABLE_FOLDERS
                        .iter()
                        .all(|folder| !component.as_str().eq_ignore_ascii_case(folder))
                })
            })
            .map(Utf8Path::to_path_buf)
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

        // If there's only one valid file in the zip, extract and analyse it
        if installer_type_counts
            .values()
            .filter(|&&count| count == 1)
            .count()
            == 1
        {
            let chosen_file_name = &possible_installer_files[0];
            nested_installer_files = BTreeSet::from([NestedInstallerFiles {
                relative_file_path: chosen_file_name.clone(),
                portable_command_alias: None,
            }]);
            if let Ok(mut chosen_file) = zip.by_name(chosen_file_name.as_str()) {
                let mut temp_file = tempfile::tempfile()?;
                io::copy(&mut chosen_file, &mut temp_file)?;
                let map = unsafe { Mmap::map(&temp_file) }?;
                let file_analyser = Analyzer::new(&map, chosen_file_name.as_str())?;
                installers = Some(
                    file_analyser
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
                        .collect::<Vec<_>>(),
                );
            }
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

    pub fn prompt(&mut self) -> Result<()> {
        if !&self.possible_installer_files.is_empty() {
            let chosen = MultiSelect::new(
                "Select the nested files",
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
            let map = unsafe { Mmap::map(&temp_file) }?;
            let file_analyser = Analyzer::new(&map, first_choice.file_name().unwrap())?;
            let nested_installer_files = chosen
                .into_iter()
                .map(|path| {
                    Ok(NestedInstallerFiles {
                        portable_command_alias: if file_analyser.installers[0].r#type
                            == Some(InstallerType::Portable)
                        {
                            Some(required_prompt(None)?)
                        } else {
                            None
                        },
                        relative_file_path: path,
                    })
                })
                .collect::<Result<BTreeSet<_>>>()?;
            self.installers = file_analyser
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
