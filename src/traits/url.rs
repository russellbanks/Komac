use reqwest::{Client, ClientBuilder, Response, redirect::Policy};
use winget_types::shared::url::DecodedUrl;

use crate::github::github_client::GITHUB_HOST;

pub trait UpgradeToHttps {
    async fn upgrade_to_https(&mut self, client: &Client);
}

impl UpgradeToHttps for DecodedUrl {
    async fn upgrade_to_https(&mut self, client: &Client) {
        const HTTP: &str = "http";
        const HTTPS: &str = "https";

        if self.scheme() == HTTP {
            self.set_scheme(HTTPS).unwrap_or_else(|()| unreachable!());
            if client
                .head(self.as_str())
                .send()
                .await
                .and_then(Response::error_for_status)
                .is_err()
            {
                self.set_scheme(HTTP).unwrap_or_else(|()| unreachable!());
            }
        }
    }
}

pub trait ConvertGitHubLatestToVersioned {
    /// Converts a vanity GitHub URL that always points to the latest release to its versioned URL by
    /// following the redirect by one hop.
    ///
    /// For example, github.com/owner/repo/releases/latest/download/file.exe to
    /// github.com/owner/repo/releases/download/v1.2.3/file.exe
    async fn convert_github_latest_to_versioned(&mut self) -> reqwest::Result<()>;
}

impl ConvertGitHubLatestToVersioned for DecodedUrl {
    async fn convert_github_latest_to_versioned(&mut self) -> reqwest::Result<()> {
        const LATEST: &str = "latest";
        const DOWNLOAD: &str = "download";
        const MAX_HOPS: u8 = 2;

        if self.host_str() != Some(GITHUB_HOST) {
            return Ok(());
        }

        if let Some(mut segments) = self.path_segments() {
            // If the 4th and 5th segments are 'latest' and 'download', it's a vanity URL
            if segments.nth(3) == Some(LATEST) && segments.next() == Some(DOWNLOAD) {
                // Create a client that will redirect only once
                let limited_redirect_client = ClientBuilder::new()
                    .redirect(Policy::limited(MAX_HOPS as usize))
                    .build()?;

                // If there was a redirect error because max hops were reached, as intended, set the
                // original vanity URL to the redirected versioned URL
                if let Err(error) = limited_redirect_client.head(self.as_str()).send().await {
                    if error.is_redirect() {
                        if let Some(final_url) = error.url() {
                            **self = final_url.clone();
                        }
                    }
                };
            }
        }
        Ok(())
    }
}
