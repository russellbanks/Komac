//! A pretend GitHub for VHS recordings, so the demo runs the real sync flow and output without a
//! token or network access.

use std::time::Duration;

use rand::random_range;
use tokio::time::sleep;

use super::service::{ForkSyncApi, MergeResult};
use crate::github::{
    Divergence, GitHubError, MergeType, Repository,
    graphql::get_sync_repositories::{RemoteFork, RemoteRepository, SyncRepositories},
};

/// Reports a fork of upstream that is a random number of commits behind, and merges it after a
/// short pause so the spinner shows.
pub struct DemoApi;

impl ForkSyncApi for DemoApi {
    async fn get_sync_repositories(
        &self,
        upstream: &Repository,
        fork: Option<&Repository>,
    ) -> Result<SyncRepositories, GitHubError> {
        let fork = fork
            .cloned()
            .unwrap_or_else(|| Repository::new("octocat", upstream.name().to_owned()));

        Ok(SyncRepositories {
            upstream: Some(remote(upstream.clone())?),
            fork: Some(RemoteFork {
                remote: remote(fork.clone())?,
                is_fork: true,
                parent: Some(upstream.to_string()),
            }),
            fork_lookup: fork,
        })
    }

    async fn compare_fork(
        &self,
        _fork: &Repository,
        _fork_branch: &str,
        _upstream: &Repository,
        _upstream_branch: &str,
    ) -> Result<Divergence, GitHubError> {
        Ok(Divergence::Behind {
            by: random_range(50..=500),
        })
    }

    async fn sync_fork(
        &self,
        _fork: &Repository,
        _branch: &str,
    ) -> Result<MergeResult, GitHubError> {
        sleep(Duration::from_secs(1)).await;

        Ok(MergeResult {
            merge_type: MergeType::FastForward,
            message: String::new(),
        })
    }
}

fn remote(repository: Repository) -> Result<RemoteRepository, GitHubError> {
    Ok(RemoteRepository {
        url: format!("https://github.com/{repository}").parse()?,
        repository,
        default_branch: "master".to_owned(),
    })
}
