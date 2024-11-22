use std::collections::{BTreeSet, HashMap};
use std::io::{Read, Seek};
use std::{io, mem};

use camino::{Utf8Path, Utf8PathBuf};
use color_eyre::eyre::Result;
use inquire::{min_length, MultiSelect, Text};
use memmap2::Mmap;
use zip::ZipArchive;

use crate::file_analyser::FileAnalyser;
use crate::manifests::installer_manifest::{NestedInstallerFiles, NestedInstallerType};
use crate::types::architecture::Architecture;
use crate::types::installer_type::InstallerType;

const VALID_NESTED_FILE_EXTENSIONS: [&str; 6] =
    ["msix", "msi", "appx", "exe", "msixbundle", "appxbundle"];

pub struct Zip<R: Read + Seek> {
    archive: ZipArchive<R>,
    pub nested_installer_type: Option<NestedInstallerType>,
    pub nested_installer_files: Option<BTreeSet<NestedInstallerFiles>>,
    pub architecture: Option<Architecture>,
    pub identified_files: Vec<Utf8PathBuf>,
}

impl<R: Read + Seek> Zip<R> {
    pub fn new(reader: R) -> Result<Self> {
        let mut zip = ZipArchive::new(reader)?;

        let identified_files = zip
            .file_names()
            .map(Utf8Path::new)
            .filter(|file_name| {
                VALID_NESTED_FILE_EXTENSIONS.iter().any(|file_extension| {
                    file_name
                        .extension()
                        .is_some_and(|extension| extension.eq_ignore_ascii_case(file_extension))
                })
            })
            .map(Utf8Path::to_path_buf)
            .collect::<Vec<_>>();

        let installer_type_counts = VALID_NESTED_FILE_EXTENSIONS
            .iter()
            .map(|file_extension| {
                (
                    file_extension,
                    zip.file_names()
                        .filter(|file_name| {
                            Utf8Path::new(file_name)
                                .extension()
                                .is_some_and(|extension| {
                                    extension.eq_ignore_ascii_case(file_extension)
                                })
                        })
                        .count(),
                )
            })
            .collect::<HashMap<_, _>>();

        let mut nested_installer_type = None;
        let mut architecture = None;

        // If there's only one valid file in the zip, extract and analyse it
        if installer_type_counts
            .values()
            .filter(|&&count| count == 1)
            .count()
            == 1
        {
            let chosen_file_name = identified_files.first().unwrap();
            if let Ok(mut chosen_file) = zip.by_name(chosen_file_name.as_str()) {
                let mut temp_file = tempfile::tempfile()?;
                io::copy(&mut chosen_file, &mut temp_file)?;
                let map = unsafe { Mmap::map(&temp_file) }?;
                let file_analyser = FileAnalyser::new(&map, chosen_file_name.as_str())?;
                nested_installer_type = file_analyser
                    .installer
                    .installer_type
                    .and_then(InstallerType::to_nested);
                architecture = Some(file_analyser.installer.architecture);
            }
            return Ok(Self {
                archive: zip,
                nested_installer_type,
                nested_installer_files: Some(BTreeSet::from([NestedInstallerFiles {
                    relative_file_path: chosen_file_name.clone(),
                    portable_command_alias: None,
                }])),
                architecture,
                identified_files,
            });
        }

        Ok(Self {
            archive: zip,
            nested_installer_type,
            nested_installer_files: None,
            architecture,
            identified_files,
        })
    }

    pub fn prompt(&mut self) -> Result<()> {
        if !&self.identified_files.is_empty() {
            let chosen = MultiSelect::new(
                "Select the nested files",
                mem::take(&mut self.identified_files),
            )
            .with_validator(min_length!(1))
            .prompt()?;
            let first_choice = chosen.first().unwrap();
            let mut temp_file = tempfile::tempfile()?;
            io::copy(
                &mut self.archive.by_name(first_choice.as_str())?,
                &mut temp_file,
            )?;
            let map = unsafe { Mmap::map(&temp_file) }?;
            let file_analyser = FileAnalyser::new(&map, first_choice.file_name().unwrap())?;
            self.nested_installer_files = Some(
                chosen
                    .into_iter()
                    .map(|path| NestedInstallerFiles {
                        portable_command_alias: if file_analyser.installer.installer_type
                            == Some(InstallerType::Portable)
                        {
                            Text::new(&format!("Portable command alias for {}:", path.as_str()))
                                .prompt()
                                .ok()
                                .filter(|alias| !alias.trim().is_empty())
                        } else {
                            None
                        },
                        relative_file_path: path,
                    })
                    .collect(),
            );
            self.architecture = Some(file_analyser.installer.architecture);
            self.nested_installer_type = file_analyser
                .installer
                .installer_type
                .and_then(InstallerType::to_nested);
        }
        Ok(())
    }
}
