//! The "Get the latest release" API.
//!
//! See <https://docs.github.com/en/rest/releases/releases#get-the-latest-release>.

use reqwest::Method;
use serde::Deserialize;

use super::{
    super::{GitHubError, client::GitHub},
    transport::SendJson,
};

#[derive(Debug, Deserialize)]
pub struct Release {
    /// The name of the tag the release was created from, e.g. `v2.18.0`.
    pub tag_name: String,
    /// The URL of the release's page on GitHub.
    pub html_url: String,
}

impl GitHub {
    /// Gets the latest published full release of a repository, ignoring drafts
    /// and prereleases.
    ///
    /// See <https://docs.github.com/rest/releases/releases#get-the-latest-release>.
    pub async fn get_latest_release(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<Release, GitHubError> {
        self.rest(
            Method::GET,
            &format!("repos/{owner}/{repo}/releases/latest"),
        )
        .send_json::<Release>()
        .await
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::Release;

    #[test]
    fn response_deserializes() {
        const RESPONSE: &str = indoc! {r#"
            {
              "url": "https://api.github.com/repos/russellbanks/Komac/releases/1",
              "html_url": "https://github.com/russellbanks/Komac/releases/tag/v2.18.0",
              "id": 1,
              "author": {
                "login": "russellbanks",
                "html_url": "https://github.com/russellbanks"
              },
              "tag_name": "v2.18.0",
              "name": "v2.18.0",
              "draft": false,
              "prerelease": false
            }
        "#};

        let release = serde_json::from_str::<Release>(RESPONSE).unwrap();

        assert_eq!(release.tag_name, "v2.18.0");
        assert_eq!(
            release.html_url,
            "https://github.com/russellbanks/Komac/releases/tag/v2.18.0"
        );
    }
}
