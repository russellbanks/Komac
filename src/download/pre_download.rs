use std::{borrow::Cow, fmt};

use camino::Utf8Path;
use const_format::formatcp;
use reqwest::{Client, ClientBuilder, Response, header::HeaderValue, redirect::Policy};
use uuid::Uuid;
use winget_types::installer::VALID_FILE_EXTENSIONS;

use crate::{github::GITHUB_HOST, manifests::Url};

/// A pre-download URL used for validating the URL and converting it into a
/// different final URL if necessary.
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct PreDownload(Url);

impl PreDownload {
    /// Creates a new pre-download from a [`Url`].
    #[inline]
    pub const fn new(url: Url) -> Self {
        Self(url)
    }

    /// Returns the pre-download's [`Url`].
    #[inline]
    pub const fn url(&self) -> &Url {
        &self.0
    }

    /// Returns a mutable reference to the pre-download's [`Url`].
    #[inline]
    pub const fn url_mut(&mut self) -> &mut Url {
        &mut self.0
    }

    /// Consumes the pre-download, returning its inner [`Url`].
    #[inline]
    pub fn into_url(self) -> Url {
        self.0
    }

    /// Gets the filename from a URL given the URL, a final redirected URL, and an optional
    /// Content-Disposition header.
    ///
    /// This works by getting the filename from the Content-Disposition header. It aims to mimic
    /// Firefox's functionality whereby the `filename*` parameter is prioritized over `filename`
    /// even if both are provided. See [Content-Disposition].
    ///
    /// If there is no Content-Disposition header or no filenames in the Content-Disposition, it falls
    /// back to getting the last part of the initial URL and then the final redirected URL if the
    /// initial URL does not have a valid file extension at the end.
    ///
    /// [Content-Disposition]: https://developer.mozilla.org/docs/Web/HTTP/Headers/Content-Disposition
    pub fn file_name<'a>(
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

    /// Upgrades the pre-download's URL to HTTPS if it is currently HTTP and the
    /// HTTPS equivalent endpoint is reachable.
    pub async fn upgrade_to_https(&mut self, client: &Client) {
        const HTTP: &str = "http";
        const HTTPS: &str = "https";

        // Only if the URL is currently HTTP
        if self.0.scheme() == HTTP {
            // Set the scheme to HTTPS
            self.0.set_scheme(HTTPS).unwrap_or_else(|()| unreachable!());

            // Check if the HTTPS equivalent is reachable
            if client
                .head((**self.0).clone())
                .send()
                .await
                .and_then(Response::error_for_status)
                .is_err()
            {
                // Change it back to HTTP if it failed
                self.0.set_scheme(HTTP).unwrap_or_else(|()| unreachable!());
            }
        }
    }

    /// Converts the pre-download's URL to a versioned GitHub URL if it is
    /// currently a GitHub URL pointing to the latest release.
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

    /// Returns the serialization of the pre-download's URL.
    #[inline]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl<T> From<T> for PreDownload
where
    T: Into<Url>,
{
    fn from(value: T) -> Self {
        Self::new(value.into())
    }
}

impl fmt::Display for PreDownload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
