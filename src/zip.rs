use std::borrow::Cow;
use std::collections::{BTreeSet, HashMap};
use std::io::{Read, Seek};
use std::{io, mem};

use camino::{Utf8Path, Utf8PathBuf};
use color_eyre::eyre::Result;
use inquire::{min_length, MultiSelect};
use memmap2::Mmap;
use zip::ZipArchive;

use crate::file_analyser::FileAnalyser;
use crate::manifests::installer_manifest::{NestedInstallerFiles, NestedInstallerType};
use crate::types::architecture::Architecture;

const VALID_NESTED_FILE_EXTENSIONS: [&str; 6] =
    ["msix", "msi", "appx", "exe", "msixbundle", "appxbundle"];

pub struct Zip {
    pub nested_installer_type: Option<NestedInstallerType>,
    pub nested_installer_files: Option<BTreeSet<NestedInstallerFiles>>,
    pub architecture: Option<Architecture>,
    pub identified_files: Vec<Utf8PathBuf>,
}

impl Zip {
    pub fn new<R: Read + Seek>(reader: R) -> Result<Self> {
        let mut zip = ZipArchive::new(reader)?;

        let identified_files = zip
            .file_names()
            .map(Utf8Path::new)
            .filter(|file_name| {
                VALID_NESTED_FILE_EXTENSIONS.iter().any(|file_extension| {
                    file_name.extension().map_or(false, |extension| {
                        extension.eq_ignore_ascii_case(file_extension)
                    })
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
                                .map_or(false, |extension| {
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
                let file_analyser =
                    FileAnalyser::new(map.as_ref(), Cow::Borrowed(chosen_file_name.as_str()))?;
                nested_installer_type = file_analyser.installer_type.to_nested();
                architecture = file_analyser.architecture;
            }
            return Ok(Self {
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
            self.nested_installer_files = Some(
                chosen
                    .into_iter()
                    .map(|path| NestedInstallerFiles {
                        relative_file_path: path,
                        portable_command_alias: None, // Prompt if portable
                    })
                    .collect(),
            );
        }
        Ok(())
    }
}
