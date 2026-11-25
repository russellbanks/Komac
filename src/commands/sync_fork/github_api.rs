//! The GitHub adapter for [`ForkSyncApi`]: calls the GitHub API and translates its responses into
//! the sync core's types.

use color_eyre::eyre::eyre;
use url::Url;

use super::{
    domain::{Divergence, MergeKind, MergeResult, RemoteFork, RemoteRepository, SyncRepositories},
    service::ForkSyncApi,
};
use crate::github::{
    CompareStatus, Comparison, GitHubError, MergeType, Repository,
    client::GitHub,
    graphql::get_sync_repositories::{
        DefaultBranchRef, ForkRepository, GetNamedFork, GetViewerFork, UpstreamRepository,
    },
};

impl ForkSyncApi for GitHub {
    type Error = GitHubError;

    async fn get_sync_repositories(
        &self,
        upstream: &Repository,
        fork: Option<&Repository>,
    ) -> Result<SyncRepositories, GitHubError> {
        match fork {
            Some(fork) => named_fork(self.get_named_fork(upstream, fork).await?, fork),
            None => viewer_fork(self.get_viewer_fork(upstream).await?, upstream.name()),
        }
    }

    async fn compare_fork(
        &self,
        fork: &Repository,
        fork_branch: &str,
        upstream: &Repository,
        upstream_branch: &str,
    ) -> Result<Divergence, GitHubError> {
        Self::compare_fork(self, fork, fork_branch, upstream, upstream_branch)
            .await
            .map(fork_divergence)
    }

    async fn sync_fork(&self, fork: &Repository, branch: &str) -> Result<MergeResult, GitHubError> {
        Self::sync_fork(self, fork, branch)
            .await
            .map(|response| merge_result(response.merge_type, response.message))
    }
}

/// Interprets a comparison whose base is the fork and whose head is upstream, flipping GitHub's
/// head-relative status so that it describes the fork.
const fn fork_divergence(comparison: Comparison) -> Divergence {
    match comparison.status {
        CompareStatus::Identical => Divergence::Identical,
        CompareStatus::Ahead => Divergence::Behind {
            by: comparison.ahead_by,
        },
        CompareStatus::Behind => Divergence::Ahead {
            by: comparison.behind_by,
        },
        CompareStatus::Diverged => Divergence::Diverged {
            behind: comparison.ahead_by,
            ahead: comparison.behind_by,
        },
    }
}

/// Translates GitHub's merge-upstream result.
fn merge_result(merge_type: MergeType, message: String) -> MergeResult {
    match merge_type {
        MergeType::Merge => MergeResult::Merged(MergeKind::Merge),
        MergeType::FastForward => MergeResult::Merged(MergeKind::FastForward),
        MergeType::None => MergeResult::NothingMerged { message },
    }
}

/// Translates a lookup of the viewer's fork, which was looked up as `<viewer>/<upstream_name>`.
fn viewer_fork(
    response: GetViewerFork,
    upstream_name: &str,
) -> Result<SyncRepositories, GitHubError> {
    Ok(SyncRepositories {
        upstream: response.upstream.map(upstream_repository).transpose()?,
        fork: response
            .viewer
            .repository
            .map(fork_repository)
            .transpose()?,
        fork_lookup: Repository::new(response.viewer.login, upstream_name.to_owned()),
    })
}

/// Translates a lookup of the fork named `fork`.
fn named_fork(response: GetNamedFork, fork: &Repository) -> Result<SyncRepositories, GitHubError> {
    Ok(SyncRepositories {
        upstream: response.upstream.map(upstream_repository).transpose()?,
        fork: response.fork.map(fork_repository).transpose()?,
        fork_lookup: fork.clone(),
    })
}

fn upstream_repository(upstream: UpstreamRepository) -> Result<RemoteRepository, GitHubError> {
    remote(
        &upstream.name_with_owner,
        upstream.url,
        upstream.default_branch_ref,
    )
}

fn fork_repository(fork: ForkRepository) -> Result<RemoteFork, GitHubError> {
    Ok(RemoteFork {
        remote: remote(&fork.name_with_owner, fork.url, fork.default_branch_ref)?,
        is_fork: fork.is_fork,
        parent: fork.parent.map(|parent| parent.name_with_owner),
    })
}

fn remote(
    name_with_owner: &str,
    url: Url,
    default_branch_ref: Option<DefaultBranchRef>,
) -> Result<RemoteRepository, GitHubError> {
    let repository = name_with_owner
        .parse::<Repository>()
        .map_err(|error| GitHubError::GraphQL(error.into()))?;

    let default_branch = default_branch_ref
        .ok_or_else(|| GitHubError::GraphQL(eyre!("{repository} has no default branch")))?
        .name;

    Ok(RemoteRepository {
        repository,
        url,
        default_branch,
    })
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::{
        Divergence, GetNamedFork, GetViewerFork, MergeKind, MergeResult, MergeType,
        fork_divergence, merge_result, named_fork, viewer_fork,
    };
    use crate::github::{Comparison, Repository};

    /// The comparison is `fork...upstream`, so GitHub's `ahead`/`behind` describe upstream
    /// relative to the fork and must be flipped to describe the fork.
    #[rstest]
    #[case("identical", 0, 0, Divergence::Identical)]
    #[case("ahead", 12, 0, Divergence::Behind { by: 12 })]
    #[case("behind", 0, 3, Divergence::Ahead { by: 3 })]
    #[case("diverged", 12, 3, Divergence::Diverged { behind: 12, ahead: 3 })]
    fn comparison_is_flipped_to_describe_the_fork(
        #[case] status: &str,
        #[case] ahead_by: u32,
        #[case] behind_by: u32,
        #[case] expected: Divergence,
    ) {
        let comparison = serde_json::from_str::<Comparison>(&format!(
            r#"{{"status":"{status}","ahead_by":{ahead_by},"behind_by":{behind_by},"total_commits":{ahead_by}}}"#
        ))
        .unwrap();

        assert_eq!(fork_divergence(comparison), expected);
    }

    #[rstest]
    #[case(MergeType::Merge, MergeResult::Merged(MergeKind::Merge))]
    #[case(MergeType::FastForward, MergeResult::Merged(MergeKind::FastForward))]
    #[case(
        MergeType::None,
        MergeResult::NothingMerged { message: "This branch is not behind the upstream".to_owned() }
    )]
    fn merge_type_is_translated(#[case] merge_type: MergeType, #[case] expected: MergeResult) {
        assert_eq!(
            merge_result(
                merge_type,
                "This branch is not behind the upstream".to_owned()
            ),
            expected
        );
    }

    const UPSTREAM: &str = r#"{
        "nameWithOwner": "microsoft/winget-pkgs",
        "url": "https://github.com/microsoft/winget-pkgs",
        "defaultBranchRef": { "name": "master" }
    }"#;

    const FORK: &str = r#"{
        "nameWithOwner": "octocat/renamed-pkgs",
        "url": "https://github.com/octocat/renamed-pkgs",
        "defaultBranchRef": { "name": "main" },
        "isFork": true,
        "parent": { "nameWithOwner": "microsoft/winget-pkgs" }
    }"#;

    fn viewer_response(fork: &str) -> GetViewerFork {
        serde_json::from_str(&format!(
            r#"{{"upstream": {UPSTREAM}, "viewer": {{"login": "octocat", "repository": {fork}}}}}"#
        ))
        .unwrap()
    }

    #[test]
    fn viewer_fork_uses_current_names_and_default_branches() {
        let repositories = viewer_fork(viewer_response(FORK), "winget-pkgs").unwrap();

        let upstream = repositories.upstream.unwrap();
        assert_eq!(
            upstream.repository,
            Repository::new("microsoft", "winget-pkgs")
        );
        assert_eq!(upstream.default_branch, "master");

        let fork = repositories.fork.unwrap();
        assert_eq!(
            fork.remote.repository,
            Repository::new("octocat", "renamed-pkgs")
        );
        assert_eq!(
            fork.remote.url.as_str(),
            "https://github.com/octocat/renamed-pkgs"
        );
        assert_eq!(fork.remote.default_branch, "main");
        assert!(fork.is_fork);
        assert_eq!(fork.parent.as_deref(), Some("microsoft/winget-pkgs"));
    }

    #[test]
    fn viewer_fork_lookup_is_login_with_upstream_name() {
        let repositories = viewer_fork(viewer_response("null"), "winget-pkgs").unwrap();

        assert!(repositories.fork.is_none());
        assert_eq!(
            repositories.fork_lookup,
            Repository::new("octocat", "winget-pkgs")
        );
    }

    #[test]
    fn named_fork_not_found() {
        let response = serde_json::from_str::<GetNamedFork>(&format!(
            r#"{{"upstream": {UPSTREAM}, "fork": null}}"#
        ))
        .unwrap();

        let repositories = named_fork(response, &Repository::new("octocat", "missing")).unwrap();

        assert!(repositories.upstream.is_some());
        assert!(repositories.fork.is_none());
        assert_eq!(
            repositories.fork_lookup,
            Repository::new("octocat", "missing")
        );
    }

    #[test]
    fn repository_without_default_branch_is_an_error() {
        let fork = FORK.replace(r#"{ "name": "main" }"#, "null");

        assert!(viewer_fork(viewer_response(&fork), "winget-pkgs").is_err());
    }
}
