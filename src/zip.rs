use crate::file_analyser::FileAnalyser;
use crate::manifests::installer_manifest::{NestedInstallerFiles, NestedInstallerType};
use crate::url_utils::VALID_FILE_EXTENSIONS;
use async_tempfile::TempFile;
use async_zip::tokio::read::seek::ZipFileReader;
use color_eyre::eyre::Result;
use inquire::{min_length, MultiSelect};
use std::collections::{BTreeSet, HashMap};
use std::mem;
use std::path::Path;
use tokio::fs::File;
use tokio::io;
use tokio_util::compat::FuturesAsyncReadCompatExt;

pub struct Zip {
    pub nested_installer_type: Option<NestedInstallerType>,
    pub nested_installer_files: Option<BTreeSet<NestedInstallerFiles>>,
    identified_files: Vec<String>,
}

impl Zip {
    pub async fn new(file: &mut File) -> Result<Self> {
        let mut zip = ZipFileReader::with_tokio(file).await?;

        let file_names = zip
            .file()
            .entries()
            .iter()
            .filter_map(|stored_entry| stored_entry.filename().as_str().ok())
            .collect::<Vec<_>>();

        let mut identified_files = file_names
            .iter()
            .filter(|file_name| {
                VALID_FILE_EXTENSIONS.iter().any(|file_extension| {
                    Path::new(file_name).extension().map_or(false, |extension| {
                        extension.eq_ignore_ascii_case(file_extension)
                    })
                })
            })
            .map(|path| (*path).to_string())
            .collect::<Vec<_>>();

        let installer_type_counts = VALID_FILE_EXTENSIONS
            .iter()
            .map(|file_extension| {
                (
                    file_extension,
                    file_names
                        .iter()
                        .filter(|file_name| {
                            Path::new(file_name).extension().map_or(false, |extension| {
                                extension.eq_ignore_ascii_case(file_extension)
                            })
                        })
                        .count(),
                )
            })
            .collect::<HashMap<_, _>>();

        // If there's only one valid file in the zip, extract and analyse it
        if installer_type_counts
            .values()
            .filter(|&&count| count == 1)
            .count()
            == 1
        {
            let mut nested_installer_type = None;
            let chosen_file_name = (*identified_files.swap_remove(0)).to_string();
            for index in 0..zip.file().entries().len() {
                if get_entry_file_name(&zip, index) == chosen_file_name {
                    let entry_reader = zip.reader_without_entry(index).await?;
                    let temp_file = TempFile::new_with_name(&chosen_file_name).await?;
                    io::copy(&mut entry_reader.compat(), &mut temp_file.open_rw().await?).await?;
                    let file_analyser =
                        FileAnalyser::new(&mut temp_file.open_ro().await?, true).await?;
                    nested_installer_type = file_analyser.installer_type.to_nested();
                    break;
                }
            }
            return Ok(Self {
                nested_installer_type,
                nested_installer_files: Some(BTreeSet::from([NestedInstallerFiles {
                    relative_file_path: chosen_file_name,
                    portable_command_alias: None,
                }])),
                identified_files: Vec::new(),
            });
        }

        Ok(Self {
            nested_installer_type: None,
            nested_installer_files: None,
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

pub fn get_entry_file_name<'reader>(
    zip: &'reader ZipFileReader<&mut File>,
    index: usize,
) -> &'reader str {
    zip.file()
        .entries()
        .get(index)
        .and_then(|stored_entry| stored_entry.filename().as_str().ok())
        .unwrap_or_default()
}
