use std::fs::File;

use anstream::stdout;
use camino::{Utf8Path, Utf8PathBuf};
use clap::Parser;
use color_eyre::{Result, eyre::ensure};
use memmap2::Mmap;
use sha2::{Digest, Sha256};
use winget_types::Sha256String;

use crate::{analysis::Analyzer, manifests::print_manifest};

/// Analyses a file and outputs information about it
#[derive(Parser)]
pub struct Analyse {
    #[arg(value_parser = is_valid_file, value_hint = clap::ValueHint::FilePath)]
    file_path: Utf8PathBuf,

    #[cfg(not(debug_assertions))]
    /// Hash the file and include it in the `InstallerSha256` field
    #[arg(long = "hash", alias = "sha256", overrides_with = "hash")]
    _no_hash: bool,

    #[cfg(not(debug_assertions))]
    /// Skip hashing the file
    #[arg(long = "no-hash", alias = "no-sha256", action = clap::ArgAction::SetFalse)]
    hash: bool,

    #[cfg(debug_assertions)]
    /// Hash the file and include it in the `InstallerSha256` field
    #[arg(long, alias = "sha256", overrides_with = "_no_hash")]
    hash: bool,

    #[cfg(debug_assertions)]
    /// Skip hashing the file
    #[arg(long = "no-hash", alias = "no-sha256")]
    _no_hash: bool,
}

impl Analyse {
    pub fn run(self) -> Result<()> {
        let file = File::open(&self.file_path)?;
        let mmap = unsafe { Mmap::map(&file) }?;
        let file_name = self
            .file_path
            .file_name()
            .unwrap_or_else(|| self.file_path.as_str());
        let mut analyser = Analyzer::new(&mmap, file_name)?;
        if self.hash {
            let sha_256 = Sha256String::from_digest(&Sha256::digest(&mmap));
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
