use std::fs::File;

use camino::Utf8Path;
use chrono::NaiveDate;
use color_eyre::eyre::Result;
use memmap2::Mmap;
use sha2::{Digest, Sha256};
use winget_types::Sha256String;

use crate::manifests::Url;

pub struct DownloadedFile {
    // As the downloaded file is a temporary file, it's stored here so that the reference stays
    // alive and the file does not get deleted. This is necessary because the memory map needs the
    // file to remain present.
    #[expect(dead_code)]
    pub file: File,
    pub url: Url,
    pub mmap: Mmap,
    pub sha_256: Sha256String,
    pub file_name: String,
    pub last_modified: Option<NaiveDate>,
}

impl DownloadedFile {
    pub fn from_local(path: &Utf8Path, url: Url) -> Result<Self> {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file) }?;
        let sha_256 = Sha256String::from_digest(&Sha256::digest(&mmap));
        let file_name = path.file_name().unwrap_or_else(|| path.as_str()).to_owned();
        Ok(Self {
            file,
            url,
            mmap,
            sha_256,
            file_name,
            last_modified: None,
        })
    }
}
