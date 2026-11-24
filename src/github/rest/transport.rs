//! Shared plumbing for GitHub REST API requests.

use reqwest::{Method, RequestBuilder, header::ACCEPT};
use serde::de::DeserializeOwned;

use super::{
    super::{GitHubError, client::GitHub},
    GITHUB_JSON_MIME, REST_API_URL, REST_API_VERSION, X_GITHUB_API_VERSION,
    error::RestError,
};

impl GitHub {
    /// Starts a REST API request with GitHub's standard headers.
    ///
    /// `path` is relative to the API root, e.g. `repos/microsoft/winget-pkgs`.
    pub(super) fn rest(&self, method: Method, path: &str) -> RequestBuilder {
        self.0
            .request(method, format!("{REST_API_URL}/{path}"))
            .header(ACCEPT, GITHUB_JSON_MIME)
            .header(X_GITHUB_API_VERSION, REST_API_VERSION)
    }
}

pub(super) trait SendJson {
    /// Sends the request and decodes a successful JSON response as `T`.
    ///
    /// An unsuccessful response becomes a [`RestError`] that keeps its HTTP status.
    async fn send_json<T: DeserializeOwned>(self) -> Result<T, GitHubError>;
}

impl SendJson for RequestBuilder {
    async fn send_json<T: DeserializeOwned>(self) -> Result<T, GitHubError> {
        let response = self.send().await?;
        let status = response.status();

        if status.is_success() {
            Ok(response.json::<T>().await?)
        } else {
            let body = response.bytes().await?;
            Err(RestError::from_body(status, &body).into())
        }
    }
}
