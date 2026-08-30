use std::fs::File;

use chrono::NaiveDate;
use winget_types::Sha256String;

use crate::download::Download;

pub struct DownloadedFile {
    pub file: File,
    pub download: Download,
    pub sha_256: Sha256String,
    pub last_modified: Option<NaiveDate>,
}
