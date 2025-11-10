mod git_file_mode;
mod object;
mod r#type;

use std::collections::BTreeSet;

pub use git_file_mode::GitFileMode;
use itertools::Itertools;
pub use object::TreeObject;
use reqwest::header::ACCEPT;
use serde::{Deserialize, Serialize};
pub use r#type::TreeType;
use winget_types::{PackageIdentifier, PackageVersion};

use super::{
    super::{GitHubError, client::GitHub, utils::PackagePath},
    GITHUB_JSON_MIME, REST_API_URL, REST_API_VERSION, X_GITHUB_API_VERSION,
};

/// A Git Tree which represents the hierarchy between files in a Git repository.
#[derive(Serialize, Deserialize)]
pub struct GitTree {
    pub sha: String,
    pub url: String,
    pub truncated: bool,
    pub tree: Vec<TreeObject>,
}

impl GitHub {
    pub async fn get_versions(
        &self,
        package_identifier: &PackageIdentifier,
    ) -> Result<BTreeSet<PackageVersion>, GitHubError> {
        self.get_all_versions(
            self.source.owner(),
            self.source.repo(),
            PackagePath::new(package_identifier, None, None),
        )
        .await
        .map_err(|_| GitHubError::PackageNonExistent(package_identifier.clone()))
    }

    /// Returns all valid package versions under a specific repository path
    ///
    /// This function inspects the Git tree at the given path in the target repository, identifies
    /// directories corresponding to version folders, and returns all versions whose entries consist
    /// entirely of file objects (i.e. no subtrees).
    async fn get_all_versions(
        &self,
        owner: &str,
        repo: &str,
        path: PackagePath,
    ) -> Result<BTreeSet<PackageVersion>, GitHubError> {
        const SEPARATOR: char = '/';

        let endpoint = format!(
            "{REST_API_URL}/repos/{owner}/{repo}/git/trees/HEAD:{path}?recursive={recursive}",
            recursive = true
        );

        let response = self
            .client
            .get(endpoint)
            .header(ACCEPT, GITHUB_JSON_MIME)
            .header(X_GITHUB_API_VERSION, REST_API_VERSION)
            .send()
            .await?
            .error_for_status()?;

        let GitTree { tree, .. } = response.json::<GitTree>().await?;

        let versions = tree
            .iter()
            .filter(|entry| entry.path.matches(SEPARATOR).count() == 1)
            .chunk_by(|entry| {
                entry
                    .path
                    .split_once(SEPARATOR)
                    .map_or(entry.path.as_str(), |(version, _rest)| version)
            })
            .into_iter()
            .filter_map(|(version, mut group)| {
                group
                    .all(|object| !object.is_tree())
                    .then(|| version.parse::<PackageVersion>().ok())?
            })
            .collect::<BTreeSet<_>>();

        if versions.is_empty() {
            Err(GitHubError::NoValidFiles { path })
        } else {
            Ok(versions)
        }
    }
}
