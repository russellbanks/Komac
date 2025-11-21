use std::fs::File;

use chrono::NaiveDate;
use winget_types::Sha256String;

use crate::manifests::Url;

pub struct DownloadedFile {
    pub file: File,
    pub url: Url,
    pub sha_256: Sha256String,
    pub file_name: String,
    pub last_modified: Option<NaiveDate>,
}
