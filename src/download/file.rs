use std::{fs::File, io, io::Read};

use camino::Utf8Path;
use chrono::NaiveDate;
use sha2::{Digest, Sha256, digest::Output};
use winget_types::Sha256String;

use crate::manifests::Url;

pub struct DownloadedFile {
    pub file: File,
    pub url: Url,
    pub sha_256: Sha256String,
    pub file_name: String,
    pub last_modified: Option<NaiveDate>,
}

impl DownloadedFile {
    pub fn from_local(path: &Utf8Path, url: Url) -> io::Result<Self> {
        let file = File::open(path)?;
        let sha_256 = Sha256String::from_digest(&sha256_digest(&file)?);
        let file_name = path.file_name().unwrap_or_else(|| path.as_str()).to_owned();
        Ok(Self {
            file,
            url,
            sha_256,
            file_name,
            last_modified: None,
        })
    }
}

pub fn sha256_digest<R: Read>(mut reader: R) -> io::Result<Output<Sha256>> {
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
