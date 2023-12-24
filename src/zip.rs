use crate::file_analyser::FileAnalyser;
use crate::manifests::installer_manifest::{InstallerType, NestedInstallerFiles};
use crate::url_utils::VALID_FILE_EXTENSIONS;
use async_tempfile::TempFile;
use async_zip::tokio::read::seek::ZipFileReader;
use color_eyre::eyre::Result;
use std::collections::{BTreeSet, HashMap};
use tokio::fs::File;
use tokio::io;
use tokio_util::compat::FuturesAsyncReadCompatExt;

pub struct Zip {
    pub nested_installer_type: Option<InstallerType>,
    pub nested_installer_files: Option<BTreeSet<NestedInstallerFiles>>,
}

impl Zip {
    pub async fn new(file: &mut File) -> Result<Zip> {
        let mut zip = ZipFileReader::with_tokio(file).await?;

        let file_names = zip
            .file()
            .entries()
            .iter()
            .flat_map(|stored_entry| stored_entry.filename().as_str().ok())
            .collect::<Vec<_>>();

        let mut identified_files = file_names
            .iter()
            .filter(|file_name| {
                VALID_FILE_EXTENSIONS
                    .into_iter()
                    .any(|file_extension| file_name.ends_with(&format!(".{file_extension}")))
            })
            .collect::<Vec<_>>();

        let installer_type_counts = VALID_FILE_EXTENSIONS
            .iter()
            .map(|file_extension| {
                (
                    file_extension,
                    file_names
                        .iter()
                        .filter(|file_name| file_name.ends_with(&format!(".{file_extension}")))
                        .count(),
                )
            })
            .collect::<HashMap<_, _>>();

        if installer_type_counts
            .values()
            .filter(|&&count| count == 1)
            .count()
            == 1
        {
            let mut nested_installer_type = None;
            let chosen_file_name = identified_files.swap_remove(0).to_string();
            for index in 0..zip.file().entries().len() {
                if get_entry_file_name(&zip, index) == chosen_file_name {
                    let entry_reader = zip.reader_without_entry(index).await?;
                    let temp_file = TempFile::new_with_name(&chosen_file_name).await?;
                    io::copy(&mut entry_reader.compat(), &mut temp_file.open_rw().await?).await?;
                    let file_analyser = FileAnalyser::new(&mut temp_file.open_ro().await?).await?;
                    nested_installer_type = Some(file_analyser.installer_type);
                    break;
                }
            }
            return Ok(Zip {
                nested_installer_type,
                nested_installer_files: Some(BTreeSet::from([NestedInstallerFiles {
                    relative_file_path: chosen_file_name.to_string(),
                    portable_command_alias: None,
                }])),
            });
        }

        Ok(Zip {
            nested_installer_type: None,
            nested_installer_files: None,
        })
    }

    pub fn prompt() {}
}

pub fn get_entry_file_name<'a>(zip: &'a ZipFileReader<&mut File>, index: usize) -> &'a str {
    zip.file()
        .entries()
        .get(index)
        .and_then(|stored_entry| stored_entry.filename().as_str().ok())
        .unwrap_or_default()
}
