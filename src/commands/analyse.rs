use crate::file_analyser::FileAnalyser;
use crate::manifests::print_manifest;
use crate::types::sha_256::Sha256String;
use anstream::stdout;
use camino::{Utf8Path, Utf8PathBuf};
use clap::Parser;
use color_eyre::Result;
use color_eyre::eyre::ensure;
use memmap2::Mmap;
use sha2::{Digest, Sha256};
use std::fs::File;

/// Analyses a file and outputs information about it
#[derive(Parser)]
pub struct Analyse {
    #[arg(value_parser = is_valid_file, value_hint = clap::ValueHint::FilePath)]
    file_path: Utf8PathBuf,

    /// Hash the file and include it in the `InstallerSha256` field
    #[arg(long, alias = "sha256")]
    hash: bool,
}

impl Analyse {
    pub fn run(self) -> Result<()> {
        let file = File::open(&self.file_path)?;
        let mmap = unsafe { Mmap::map(&file) }?;
        let file_name = self
            .file_path
            .file_name()
            .unwrap_or_else(|| self.file_path.as_str());
        let mut analyser = FileAnalyser::new(&mmap, file_name)?;
        if self.hash {
            let sha_256 = Sha256String::from_hasher(&Sha256::digest(&mmap))?;
            for installer in &mut analyser.installers {
                installer.sha_256 = sha_256.clone();
            }
        }
        let yaml = match analyser.installers.as_slice() {
            [installer] => serde_yaml::to_string(installer)?,
            installers => serde_yaml::to_string(installers)?,
        };
        let mut lock = stdout().lock();
        print_manifest(&mut lock, &yaml);
        Ok(())
    }
}

fn is_valid_file(path: &str) -> Result<Utf8PathBuf> {
    let path = Utf8Path::new(path);
    ensure!(path.exists(), "{path} does not exist");
    ensure!(path.is_file(), "{path} is not a file");
    Ok(path.to_path_buf())
}
