mod downloader;
mod downloads;
mod file;
mod pre_download;

use std::fmt;

use chrono::{DateTime, NaiveDate};
pub use downloader::Downloader;
pub use downloads::Downloads;
pub use file::DownloadedFile;
pub use pre_download::PreDownload;
use reqwest::{Response, header::LAST_MODIFIED};

use crate::manifests::Url;

#[derive(Debug)]
pub struct Download {
    url: Url,
    file_name: String,
    pub response: Option<Response>,
}

impl Download {
    /// Creates a new [`Download`] from a [`Url`], a file name, and a [`Response`].
    #[inline]
    pub const fn new(url: Url, file_name: String, response: Response) -> Self {
        Self {
            url,
            file_name,
            response: Some(response),
        }
    }

    /// Returns a reference to the download's [`Url`].
    #[inline]
    pub const fn url(&self) -> &Url {
        &self.url
    }

    /// Returns a mutable reference to the download's [`Url`].
    #[inline]
    pub const fn url_mut(&mut self) -> &mut Url {
        &mut self.url
    }

    /// Returns the last modified response header as a [`NaiveDate`].
    pub fn last_modified(&self) -> Option<NaiveDate> {
        self.response.as_ref().and_then(|response| {
            response
                .headers()
                .get(LAST_MODIFIED)
                .and_then(|last_modified| last_modified.to_str().ok())
                .and_then(|last_modified| DateTime::parse_from_rfc2822(last_modified).ok())
                .map(|date_time| date_time.date_naive())
        })
    }

    /// Returns the content length of the response, if known.
    pub fn content_length(&self) -> Option<u64> {
        self.response.as_ref().and_then(Response::content_length)
    }
}

impl fmt::Display for Download {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.url.fmt(f)
    }
}
