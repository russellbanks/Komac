use std::fs::File;

use chrono::NaiveDate;
use memmap2::Mmap;
use winget_types::{Sha256String, url::DecodedUrl};

pub struct DownloadedFile {
    // As the downloaded file is a temporary file, it's stored here so that the reference stays
    // alive and the file does not get deleted. This is necessary because the memory map needs the
    // file to remain present.
    #[expect(dead_code)]
    pub file: File,
    pub url: DecodedUrl,
    pub mmap: Mmap,
    pub sha_256: Sha256String,
    pub file_name: String,
    pub last_modified: Option<NaiveDate>,
}
