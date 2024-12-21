use crate::file_analyser::FileAnalyser;
use crate::manifests::print_manifest;
use crate::types::sha_256::Sha256String;
use anstream::stdout;
use camino::{Utf8Path, Utf8PathBuf};
use clap::Parser;
use color_eyre::eyre::ensure;
use color_eyre::Result;
use memmap2::Mmap;
use sha2::{Digest, Sha256};
use std::fs::File;

/// 分析文件并输出有关信息
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
    ensure!(path.exists(), "{path} 不存在");
    ensure!(path.is_file(), "{path} 不是一个文件");
    Ok(path.to_path_buf())
}
