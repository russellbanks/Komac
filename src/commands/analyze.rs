use std::{
    fs::File,
    io,
    io::{Read, Seek, SeekFrom},
};

use anstream::stdout;
use camino::{Utf8Path, Utf8PathBuf};
use clap::Parser;
use color_eyre::{Result, eyre::ensure};
use sha2::{Digest, Sha256, digest::Output};
use winget_types::Sha256String;

use crate::{analysis::Analyzer, i18n::fl, manifests::print_manifest};

#[derive(Parser)]
#[command(about = fl!("command-analyze"))]
#[clap(visible_alias = "analyse")]
pub struct Analyze {
    #[arg(value_parser = is_valid_file, value_hint = clap::ValueHint::FilePath)]
    file_path: Utf8PathBuf,

    #[cfg(not(debug_assertions))]
    #[arg(
        long = "hash",
        alias = "sha256",
        overrides_with = "hash",
        help = fl!("arg-analyze-hash"),
    )]
    _no_hash: bool,

    #[cfg(not(debug_assertions))]
    #[arg(
        long = "no-hash",
        alias = "no-sha256",
        action = clap::ArgAction::SetFalse,
        help = fl!("arg-analyze-no-hash"),
    )]
    hash: bool,

    #[cfg(debug_assertions)]
    #[arg(long, alias = "sha256", overrides_with = "_no_hash", help = fl!("arg-analyze-hash"))]
    hash: bool,

    #[cfg(debug_assertions)]
    #[arg(long = "no-hash", alias = "no-sha256", help = fl!("arg-analyze-no-hash"))]
    _no_hash: bool,
}

impl Analyze {
    pub fn run(self) -> Result<()> {
        let mut file = File::open(&self.file_path)?;
        let file_name = self
            .file_path
            .file_name()
            .unwrap_or_else(|| self.file_path.as_str());
        let mut installers = Analyzer::new(&mut file, file_name)?.installers;
        if self.hash {
            file.seek(SeekFrom::Start(0))?;
            let sha_256 = Sha256String::from_digest(&sha256_digest(file)?);
            for installer in &mut installers {
                installer.sha_256 = sha_256.clone();
            }
        }
        let yaml = match installers.as_slice() {
            [installer] => serde_yaml::to_string(installer)?,
            installers => serde_yaml::to_string(installers)?,
        };
        let mut lock = stdout().lock();
        print_manifest(&mut lock, &yaml);
        Ok(())
    }
}

fn sha256_digest<R: Read>(mut reader: R) -> io::Result<Output<Sha256>> {
    let mut digest = Sha256::new();
    let mut buffer = [0; 1 << 13];

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }

    Ok(digest.finalize())
}

fn is_valid_file(path: &str) -> Result<Utf8PathBuf> {
    let path = Utf8Path::new(path);
    ensure!(path.exists(), "{path} does not exist");
    ensure!(path.is_file(), "{path} is not a file");
    Ok(path.to_path_buf())
}
