use std::{
    collections::{BTreeSet, HashMap},
    io,
    io::{Read, Seek},
    mem,
};

use camino::{Utf8Path, Utf8PathBuf};
use color_eyre::eyre::Result;
use inquire::{MultiSelect, Text, min_length};
use memmap2::Mmap;
use winget_types::installer::{Installer, InstallerType, NestedInstallerFiles};
use zip::ZipArchive;

use crate::{file_analyser::FileAnalyser, prompts::handle_inquire_error};

const VALID_NESTED_FILE_EXTENSIONS: [&str; 6] =
    ["msix", "msi", "appx", "exe", "msixbundle", "appxbundle"];

const MACOS_X_FOLDER: &str = "__MACOSX";

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
                file_name
                    .components()
                    .all(|component| component.as_str() != MACOS_X_FOLDER)
            })
            .map(Utf8Path::to_path_buf)
            .collect::<Vec<_>>();

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

        let mut nested_installer_files = None;
        let mut installers = None;

        // If there's only one valid file in the zip, extract and analyse it
        if installer_type_counts
            .values()
            .filter(|&&count| count == 1)
            .count()
            == 1
        {
            let chosen_file_name = &possible_installer_files[0];
            nested_installer_files = Some(BTreeSet::from([NestedInstallerFiles {
                relative_file_path: chosen_file_name.clone(),
                portable_command_alias: None,
            }]));
            if let Ok(mut chosen_file) = zip.by_name(chosen_file_name.as_str()) {
                let mut temp_file = tempfile::tempfile()?;
                io::copy(&mut chosen_file, &mut temp_file)?;
                let map = unsafe { Mmap::map(&temp_file) }?;
                let file_analyser = FileAnalyser::new(&map, chosen_file_name.as_str())?;
                installers = Some(
                    file_analyser
                        .installers
                        .into_iter()
                        .map(|installer| Installer {
                            r#type: Some(InstallerType::Zip),
                            nested_installer_type: installer
                                .r#type
                                .and_then(InstallerType::to_nested),
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
            let file_analyser = FileAnalyser::new(&map, first_choice.file_name().unwrap())?;
            let nested_installer_files = Some(
                chosen
                    .into_iter()
                    .map(|path| {
                        Ok(NestedInstallerFiles {
                            portable_command_alias: if file_analyser.installers[0].r#type
                                == Some(InstallerType::Portable)
                            {
                                Some(
                                    Text::new(&format!(
                                        "Portable command alias for {}:",
                                        path.as_str()
                                    ))
                                    .prompt()
                                    .map_err(handle_inquire_error)?,
                                )
                                .filter(|alias| !alias.trim().is_empty())
                            } else {
                                None
                            },
                            relative_file_path: path,
                        })
                    })
                    .collect::<Result<BTreeSet<_>>>()?,
            );
            self.installers = file_analyser
                .installers
                .into_iter()
                .map(|installer| Installer {
                    nested_installer_type: installer.r#type.and_then(InstallerType::to_nested),
                    nested_installer_files: nested_installer_files.clone(),
                    ..installer
                })
                .collect();
        }
        Ok(())
    }
}
