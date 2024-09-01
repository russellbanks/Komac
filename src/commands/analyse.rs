use crate::file_analyser::FileAnalyser;
use crate::manifests::print_manifest;
use crate::types::sha_256::Sha256String;
use anstream::stdout;
use camino::{Utf8Path, Utf8PathBuf};
use clap::Parser;
use color_eyre::eyre::bail;
use color_eyre::Result;
use memmap2::Mmap;
use sha2::{Digest, Sha256};
use std::fs::File;

/// Analyses a file and outputs information about it
#[derive(Parser)]
pub struct Analyse {
    #[arg(value_parser = is_valid_file, value_hint = clap::ValueHint::FilePath)]
    file_path: Utf8PathBuf,
}

impl Analyse {
    pub fn run(self) -> Result<()> {
        let file = File::open(&self.file_path)?;
        let mmap = unsafe { Mmap::map(&file) }?;
        let file_name = self
            .file_path
            .file_name()
            .unwrap_or_else(|| self.file_path.as_str());
        let analyser = FileAnalyser::new(&mmap, file_name)?;
        let mut installer = analyser.installer;
        installer.installer_sha_256 = Sha256String::from_hasher(&Sha256::digest(&mmap))?;
        let mut lock = stdout().lock();
        print_manifest(&mut lock, &serde_yaml::to_string(&installer)?);
        Ok(())
    }
}

fn is_valid_file(path: &str) -> Result<Utf8PathBuf> {
    let path = Utf8Path::new(path);
    if !path.exists() {
        bail!("{path} does not exist")
    }
    if !path.is_file() {
        bail!("{path} is not a file")
    }
    Ok(path.to_path_buf())
}
