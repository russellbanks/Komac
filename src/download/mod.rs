mod downloader;
mod file;

use std::{borrow::Cow, fmt};

use camino::Utf8Path;
use const_format::formatcp;
pub use downloader::Downloader;
pub use file::DownloadedFile;
use reqwest::{Client, ClientBuilder, Response, header::HeaderValue, redirect::Policy};
use uuid::Uuid;
use winget_types::installer::VALID_FILE_EXTENSIONS;

use crate::{github::GITHUB_HOST, manifests::Url};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct Download(Url);

impl Download {
    #[inline]
    pub const fn new(url: Url) -> Self {
        Self(url)
    }

    #[inline]
    pub const fn url(&self) -> &Url {
        &self.0
    }

    pub fn into_url(self) -> Url {
        self.0
    }

    /// Gets the filename from a URL given the URL, a final redirected URL, and an optional
    /// Content-Disposition header.
    ///
    /// This works by getting the filename from the Content-Disposition header. It aims to mimic
    /// Firefox's functionality whereby the `filename*` parameter is prioritised over `filename`
    /// even if both are provided. See [Content-Disposition].
    ///
    /// If there is no Content-Disposition header or no filenames in the Content-Disposition, it falls
    /// back to getting the last part of the initial URL and then the final redirected URL if the
    /// initial URL does not have a valid file extension at the end.
    ///
    /// [Content-Disposition]: https://developer.mozilla.org/docs/Web/HTTP/Headers/Content-Disposition
    fn file_name<'a>(
        &'a self,
        final_url: &'a url::Url,
        content_disposition: Option<&'a HeaderValue>,
    ) -> Cow<'a, str> {
        const FILENAME: &str = "filename";
        const FILENAME_EXT: &str = formatcp!("{FILENAME}*");

        if let Some(content_disposition) = content_disposition
            && let Ok(content_disposition) = content_disposition.to_str()
        {
            let mut sections = content_disposition.split(';');
            let _disposition = sections.next(); // Skip the disposition type
            let filenames = sections
                .filter_map(|section| {
                    section
                        .split_once('=')
                        .map(|(key, value)| (key.trim(), value.trim().trim_matches('"').trim()))
                        .filter(|(key, value)| key.starts_with(FILENAME) && !value.is_empty())
                })
                .collect::<Vec<_>>();

            let filename = filenames
                .iter()
                .find_map(|&(key, value)| (key == FILENAME_EXT).then_some(value))
                .or_else(|| {
                    filenames
                        .into_iter()
                        .find_map(|(key, value)| (key == FILENAME).then_some(value))
                });
            if let Some(filename) = filename {
                return Cow::Borrowed(filename);
            }
        }

        // Fallback if there is no Content-Disposition header or no filenames in Content-Disposition
        self.0
            .path_segments()
            .and_then(|mut segments| segments.next_back())
            .filter(|last_segment| {
                Utf8Path::new(last_segment)
                    .extension()
                    .is_some_and(|extension| VALID_FILE_EXTENSIONS.contains(&extension))
            })
            .or_else(|| {
                final_url
                    .path_segments()
                    .and_then(|mut segments| segments.next_back())
            })
            .map_or_else(|| Cow::Owned(Uuid::new_v4().to_string()), Cow::Borrowed)
    }

    pub async fn upgrade_to_https(&mut self, client: &Client) {
        const HTTP: &str = "http";
        const HTTPS: &str = "https";

        if self.0.scheme() == HTTP {
            self.0.set_scheme(HTTPS).unwrap_or_else(|()| unreachable!());

            if client
                .head((**self.0).clone())
                .send()
                .await
                .and_then(Response::error_for_status)
                .is_err()
            {
                self.0.set_scheme(HTTP).unwrap_or_else(|()| unreachable!());
            }
        }
    }

    pub async fn convert_to_github_versioned(&mut self) -> reqwest::Result<()> {
        const LATEST: &str = "latest";
        const DOWNLOAD: &str = "download";
        const MAX_HOPS: u8 = 2;

        if self.0.host_str() != Some(GITHUB_HOST) {
            return Ok(());
        }

        if let Some(mut segments) = self.0.path_segments() {
            // If the 4th and 5th segments are 'latest' and 'download', it's a vanity URL
            if segments.nth(3) == Some(LATEST) && segments.next() == Some(DOWNLOAD) {
                // Create a client that will redirect only once
                let limited_redirect_client = ClientBuilder::new()
                    .redirect(Policy::limited(MAX_HOPS as usize))
                    .build()?;

                // If there was a redirect error because max hops were reached, as intended, set the
                // original vanity URL to the redirected versioned URL
                if let Err(error) = limited_redirect_client.head(self.as_str()).send().await
                    && error.is_redirect()
                    && let Some(final_url) = error.url()
                {
                    **self.0 = final_url.clone();
                }
            }
        }
        Ok(())
    }

    /// Returns the serialization of the download's URL.
    #[inline]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for Download {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> From<T> for Download
where
    T: Into<Url>,
{
    fn from(value: T) -> Self {
        Self::new(value.into())
    }
}
