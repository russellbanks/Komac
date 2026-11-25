//! The sync use case: resolve and validate the fork, compare it with upstream, and merge only when
//! upstream has commits the fork lacks. It talks to GitHub through [`ForkSyncApi`] and prints
//! nothing, so it runs the same against GitHub, the VHS demo, and test fakes.

use thiserror::Error;
use tracing::warn;

use super::summary::{Commits, MergeSummary};
use crate::github::{
    Divergence, GitHubError, MergeType, Repository,
    client::GitHub,
    graphql::get_sync_repositories::{RemoteRepository, SyncRepositories},
};

/// The GitHub operations a sync needs.
pub trait ForkSyncApi {
    /// Gets upstream and the fork, which is the user's repository with upstream's name unless
    /// `fork` names it.
    async fn get_sync_repositories(
        &self,
        upstream: &Repository,
        fork: Option<&Repository>,
    ) -> Result<SyncRepositories, GitHubError>;

    /// Describes how `fork_branch` in `fork` relates to `upstream_branch` in `upstream`.
    async fn compare_fork(
        &self,
        fork: &Repository,
        fork_branch: &str,
        upstream: &Repository,
        upstream_branch: &str,
    ) -> Result<Divergence, GitHubError>;

    /// Merges upstream into `branch` of `fork`.
    async fn sync_fork(&self, fork: &Repository, branch: &str) -> Result<MergeResult, GitHubError>;
}

/// What GitHub did when asked to merge upstream into the fork.
#[derive(Debug)]
pub struct MergeResult {
    pub merge_type: MergeType,
    /// GitHub's explanation, which says why nothing was merged when `merge_type` is `None`.
    pub message: String,
}

impl ForkSyncApi for GitHub {
    async fn get_sync_repositories(
        &self,
        upstream: &Repository,
        fork: Option<&Repository>,
    ) -> Result<SyncRepositories, GitHubError> {
        Self::get_sync_repositories(self, upstream, fork).await
    }

    async fn compare_fork(
        &self,
        fork: &Repository,
        fork_branch: &str,
        upstream: &Repository,
        upstream_branch: &str,
    ) -> Result<Divergence, GitHubError> {
        Self::compare_fork(self, fork, fork_branch, upstream, upstream_branch).await
    }

    async fn sync_fork(&self, fork: &Repository, branch: &str) -> Result<MergeResult, GitHubError> {
        Self::sync_fork(self, fork, branch)
            .await
            .map(|response| MergeResult {
                merge_type: response.merge_type,
                message: response.message,
            })
    }
}

/// Shows that a merge is in progress.
pub trait Progress {
    /// Starts showing that `summary` is being merged, until the returned guard is dropped.
    fn merging(&self, summary: &MergeSummary<&Repository, &Repository>) -> impl Sized;
}

/// What to sync.
pub struct SyncRequest<'a> {
    pub upstream: &'a Repository,
    /// The fork, if it was named rather than being the user's repository with upstream's name.
    pub fork: Option<Repository>,
    /// The branch to sync, if not the default branch.
    pub branch: Option<&'a str>,
}

/// What a sync found or did.
#[derive(Debug, PartialEq, Eq)]
pub enum SyncOutcome {
    /// The fork already matches upstream.
    UpToDate,
    /// The fork has commits upstream lacks, and upstream has nothing new to merge.
    ForkAhead(Commits),
    /// Upstream's commits were merged into the fork.
    Merged {
        commits: Commits,
        merge_type: MergeType,
    },
    /// GitHub merged nothing even though the comparison found upstream commits, e.g. because the
    /// fork was synced in between. The message is GitHub's explanation.
    NothingMerged { message: String },
}

/// The repositories that were synced and the outcome.
#[derive(Debug)]
pub struct SyncReport {
    pub upstream: RemoteRepository,
    pub fork: RemoteRepository,
    pub outcome: SyncOutcome,
}

/// Why a sync failed.
#[derive(Debug, Error)]
pub enum SyncError {
    #[error("{0} was not found, or the token cannot access it")]
    UpstreamNotFound(Repository),
    #[error("{fork} was not found. Fork {upstream} on GitHub first, or name your fork with --fork")]
    ForkNotFound {
        fork: Repository,
        upstream: Repository,
    },
    #[error("{fork} is not a fork, so it cannot be synced with {upstream}")]
    NotAFork {
        fork: Repository,
        upstream: Repository,
    },
    #[error(
        "{fork} is a fork of {parent}, not {upstream}. GitHub syncs a fork only from its parent"
    )]
    ForkOfOtherRepository {
        fork: Repository,
        parent: String,
        upstream: Repository,
    },
    #[error("{fork} is a fork, but the token cannot access its parent to check it is {upstream}")]
    ParentInaccessible {
        fork: Repository,
        upstream: Repository,
    },
    #[error("Failed to merge {summary}")]
    Merge {
        summary: String,
        source: GitHubError,
    },
    #[error(transparent)]
    GitHub(#[from] GitHubError),
}

/// Syncs the fork with upstream, merging only when upstream has commits the fork lacks.
pub async fn sync<A: ForkSyncApi>(
    api: &A,
    request: &SyncRequest<'_>,
    progress: &impl Progress,
) -> Result<SyncReport, SyncError> {
    let Target { upstream, fork } = validate(
        api.get_sync_repositories(request.upstream, request.fork.as_ref())
            .await?,
        request.upstream,
    )?;

    let branches = Branches::new(
        request.branch,
        &fork.default_branch,
        &upstream.default_branch,
    );

    let divergence = api
        .compare_fork(
            &fork.repository,
            branches.fork,
            &upstream.repository,
            branches.upstream,
        )
        .await?;

    let commits = match divergence {
        Divergence::Identical => {
            return Ok(SyncReport {
                upstream,
                fork,
                outcome: SyncOutcome::UpToDate,
            });
        }
        Divergence::Ahead { by } => {
            return Ok(SyncReport {
                upstream,
                fork,
                outcome: SyncOutcome::ForkAhead(Commits(by)),
            });
        }
        Divergence::Behind { by } => Commits(by),
        Divergence::Diverged { behind, ahead } => {
            warn!(
                "{} has diverged from {}: it has {} that {} does not. Syncing creates a merge commit, and fails if the changes conflict",
                fork.repository,
                upstream.repository,
                Commits(ahead),
                upstream.repository,
            );
            Commits(behind)
        }
    };

    let summary = MergeSummary {
        commits,
        upstream: &upstream.repository,
        fork: &fork.repository,
    };

    let result = {
        let _progress = progress.merging(&summary);
        api.sync_fork(&fork.repository, branches.fork)
            .await
            .map_err(|source| SyncError::Merge {
                summary: summary.to_string(),
                source,
            })?
    };

    let outcome = match result.merge_type {
        MergeType::Merge | MergeType::FastForward => SyncOutcome::Merged {
            commits,
            merge_type: result.merge_type,
        },
        MergeType::None => SyncOutcome::NothingMerged {
            message: result.message,
        },
    };

    Ok(SyncReport {
        upstream,
        fork,
        outcome,
    })
}

/// Upstream and a fork of it that can be synced.
#[derive(Debug)]
struct Target {
    upstream: RemoteRepository,
    fork: RemoteRepository,
}

/// Checks that both repositories exist and that the fork's parent is upstream, since GitHub syncs
/// a fork only from its direct parent.
fn validate(repositories: SyncRepositories, upstream: &Repository) -> Result<Target, SyncError> {
    let SyncRepositories {
        upstream: found_upstream,
        fork,
        fork_lookup,
    } = repositories;

    let found_upstream =
        found_upstream.ok_or_else(|| SyncError::UpstreamNotFound(upstream.clone()))?;
    let upstream = found_upstream.repository.clone();

    let fork = fork.ok_or_else(|| SyncError::ForkNotFound {
        fork: fork_lookup,
        upstream: upstream.clone(),
    })?;
    let fork_repository = fork.remote.repository.clone();

    if !fork.is_fork {
        return Err(SyncError::NotAFork {
            fork: fork_repository,
            upstream,
        });
    }

    match fork.parent {
        Some(parent) if upstream.matches_full_name(&parent) => Ok(Target {
            upstream: found_upstream,
            fork: fork.remote,
        }),
        Some(parent) => Err(SyncError::ForkOfOtherRepository {
            fork: fork_repository,
            parent,
            upstream,
        }),
        None => Err(SyncError::ParentInaccessible {
            fork: fork_repository,
            upstream,
        }),
    }
}

/// The branch to sync in the fork, and the upstream branch it is compared with.
#[derive(Debug, PartialEq, Eq)]
struct Branches<'a> {
    fork: &'a str,
    upstream: &'a str,
}

impl<'a> Branches<'a> {
    /// Uses each repository's default branch, unless a branch was given.
    ///
    /// A given branch is compared with upstream's branch of the same name, matching
    /// `gh repo sync --branch`. GitHub's API docs don't say which upstream branch merge-upstream
    /// uses for a non-default branch.
    fn new(branch: Option<&'a str>, fork_default: &'a str, upstream_default: &'a str) -> Self {
        match branch {
            Some(branch) => Self {
                fork: branch,
                upstream: branch,
            },
            None => Self {
                fork: fork_default,
                upstream: upstream_default,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use color_eyre::eyre::eyre;
    use rstest::rstest;

    use super::{
        Branches, Commits, Divergence, ForkSyncApi, GitHubError, MergeResult, MergeSummary,
        MergeType, Progress, SyncError, SyncOutcome, SyncRequest, sync, validate,
    };
    use crate::github::{
        Repository,
        graphql::get_sync_repositories::{RemoteFork, RemoteRepository, SyncRepositories},
    };

    fn upstream() -> Repository {
        Repository::new("microsoft", "winget-pkgs")
    }

    fn remote(owner: &str, name: &str, default_branch: &str) -> RemoteRepository {
        RemoteRepository {
            repository: Repository::new(owner.to_owned(), name.to_owned()),
            url: format!("https://github.com/{owner}/{name}")
                .parse()
                .unwrap(),
            default_branch: default_branch.to_owned(),
        }
    }

    fn fork_of(parent: Option<&str>) -> RemoteFork {
        RemoteFork {
            remote: remote("octocat", "winget-pkgs", "main"),
            is_fork: true,
            parent: parent.map(str::to_owned),
        }
    }

    fn repositories(
        upstream: Option<RemoteRepository>,
        fork: Option<RemoteFork>,
    ) -> SyncRepositories {
        SyncRepositories {
            upstream,
            fork,
            fork_lookup: Repository::new("octocat", "winget-pkgs"),
        }
    }

    /// A fork of upstream whose default branch is `main`, while upstream's is `master`.
    fn valid_repositories() -> SyncRepositories {
        repositories(
            Some(remote("microsoft", "winget-pkgs", "master")),
            Some(fork_of(Some("microsoft/winget-pkgs"))),
        )
    }

    /// A scripted GitHub that records the branches it is asked to compare and merge.
    struct FakeApi {
        repositories: RefCell<Option<SyncRepositories>>,
        divergence: Divergence,
        merge: Result<MergeType, &'static str>,
        compared: RefCell<Option<(String, String)>>,
        merged: RefCell<Option<String>>,
    }

    impl FakeApi {
        fn new(divergence: Divergence, merge: Result<MergeType, &'static str>) -> Self {
            Self::with_repositories(valid_repositories(), divergence, merge)
        }

        fn with_repositories(
            repositories: SyncRepositories,
            divergence: Divergence,
            merge: Result<MergeType, &'static str>,
        ) -> Self {
            Self {
                repositories: RefCell::new(Some(repositories)),
                divergence,
                merge,
                compared: RefCell::new(None),
                merged: RefCell::new(None),
            }
        }
    }

    impl ForkSyncApi for FakeApi {
        async fn get_sync_repositories(
            &self,
            _upstream: &Repository,
            _fork: Option<&Repository>,
        ) -> Result<SyncRepositories, GitHubError> {
            Ok(self.repositories.borrow_mut().take().unwrap())
        }

        async fn compare_fork(
            &self,
            _fork: &Repository,
            fork_branch: &str,
            _upstream: &Repository,
            upstream_branch: &str,
        ) -> Result<Divergence, GitHubError> {
            *self.compared.borrow_mut() =
                Some((fork_branch.to_owned(), upstream_branch.to_owned()));
            Ok(self.divergence)
        }

        async fn sync_fork(
            &self,
            _fork: &Repository,
            branch: &str,
        ) -> Result<MergeResult, GitHubError> {
            *self.merged.borrow_mut() = Some(branch.to_owned());
            self.merge
                .map(|merge_type| MergeResult {
                    merge_type,
                    message: "GitHub's explanation".to_owned(),
                })
                .map_err(|error| GitHubError::GraphQL(eyre!(error)))
        }
    }

    /// Records each merge it is asked to show.
    #[derive(Default)]
    struct RecordingProgress(RefCell<Vec<String>>);

    impl Progress for RecordingProgress {
        fn merging(&self, summary: &MergeSummary<&Repository, &Repository>) -> impl Sized {
            self.0.borrow_mut().push(summary.to_string());
        }
    }

    fn request<'a>(upstream: &'a Repository, branch: Option<&'a str>) -> SyncRequest<'a> {
        SyncRequest {
            upstream,
            fork: None,
            branch,
        }
    }

    #[tokio::test]
    async fn up_to_date_fork_is_not_merged() {
        let api = FakeApi::new(Divergence::Identical, Ok(MergeType::None));
        let progress = RecordingProgress::default();

        let report = sync(&api, &request(&upstream(), None), &progress)
            .await
            .unwrap();

        assert_eq!(report.outcome, SyncOutcome::UpToDate);
        assert!(api.merged.borrow().is_none());
        assert!(progress.0.borrow().is_empty());
    }

    #[tokio::test]
    async fn fork_ahead_is_not_merged() {
        let api = FakeApi::new(Divergence::Ahead { by: 3 }, Ok(MergeType::None));

        let report = sync(
            &api,
            &request(&upstream(), None),
            &RecordingProgress::default(),
        )
        .await
        .unwrap();

        assert_eq!(report.outcome, SyncOutcome::ForkAhead(Commits(3)));
        assert!(api.merged.borrow().is_none());
    }

    #[tokio::test]
    async fn behind_fork_merges_its_default_branch() {
        let api = FakeApi::new(Divergence::Behind { by: 5 }, Ok(MergeType::FastForward));
        let progress = RecordingProgress::default();

        let report = sync(&api, &request(&upstream(), None), &progress)
            .await
            .unwrap();

        assert_eq!(
            report.outcome,
            SyncOutcome::Merged {
                commits: Commits(5),
                merge_type: MergeType::FastForward
            }
        );
        assert_eq!(
            *api.compared.borrow(),
            Some(("main".to_owned(), "master".to_owned()))
        );
        assert_eq!(api.merged.borrow().as_deref(), Some("main"));
        assert_eq!(
            *progress.0.borrow(),
            ["5 commits from microsoft/winget-pkgs into octocat/winget-pkgs"]
        );
        assert_eq!(
            report.fork.repository,
            Repository::new("octocat", "winget-pkgs")
        );
        assert_eq!(report.upstream.repository, upstream());
    }

    #[tokio::test]
    async fn diverged_fork_merges_upstream_commits() {
        let api = FakeApi::new(
            Divergence::Diverged {
                behind: 4,
                ahead: 2,
            },
            Ok(MergeType::Merge),
        );

        let report = sync(
            &api,
            &request(&upstream(), None),
            &RecordingProgress::default(),
        )
        .await
        .unwrap();

        assert_eq!(
            report.outcome,
            SyncOutcome::Merged {
                commits: Commits(4),
                merge_type: MergeType::Merge
            }
        );
    }

    #[tokio::test]
    async fn nothing_merged_keeps_github_explanation() {
        let api = FakeApi::new(Divergence::Behind { by: 1 }, Ok(MergeType::None));

        let report = sync(
            &api,
            &request(&upstream(), None),
            &RecordingProgress::default(),
        )
        .await
        .unwrap();

        assert_eq!(
            report.outcome,
            SyncOutcome::NothingMerged {
                message: "GitHub's explanation".to_owned()
            }
        );
    }

    #[tokio::test]
    async fn given_branch_is_compared_and_merged() {
        let api = FakeApi::new(Divergence::Behind { by: 1 }, Ok(MergeType::FastForward));

        sync(
            &api,
            &request(&upstream(), Some("release")),
            &RecordingProgress::default(),
        )
        .await
        .unwrap();

        assert_eq!(
            *api.compared.borrow(),
            Some(("release".to_owned(), "release".to_owned()))
        );
        assert_eq!(api.merged.borrow().as_deref(), Some("release"));
    }

    #[tokio::test]
    async fn invalid_fork_stops_before_comparing() {
        let api = FakeApi::with_repositories(
            repositories(Some(remote("microsoft", "winget-pkgs", "master")), None),
            Divergence::Behind { by: 1 },
            Ok(MergeType::FastForward),
        );

        let error = sync(
            &api,
            &request(&upstream(), None),
            &RecordingProgress::default(),
        )
        .await
        .unwrap_err();

        assert!(matches!(error, SyncError::ForkNotFound { .. }));
        assert!(api.compared.borrow().is_none());
    }

    #[tokio::test]
    async fn failed_merge_says_what_was_being_merged() {
        let api = FakeApi::new(Divergence::Behind { by: 5 }, Err("merge conflict"));

        let error = sync(
            &api,
            &request(&upstream(), None),
            &RecordingProgress::default(),
        )
        .await
        .unwrap_err();

        assert_eq!(
            error.to_string(),
            "Failed to merge 5 commits from microsoft/winget-pkgs into octocat/winget-pkgs"
        );
        assert!(matches!(error, SyncError::Merge { .. }));
    }

    #[rstest]
    #[case("microsoft/winget-pkgs")]
    #[case("Microsoft/Winget-Pkgs")]
    fn fork_of_upstream_is_valid(#[case] parent: &str) {
        let target = validate(
            repositories(
                Some(remote("microsoft", "winget-pkgs", "master")),
                Some(fork_of(Some(parent))),
            ),
            &upstream(),
        )
        .unwrap();

        assert_eq!(target.upstream.repository, upstream());
        assert_eq!(
            target.fork.repository,
            Repository::new("octocat", "winget-pkgs")
        );
    }

    #[test]
    fn missing_upstream() {
        let error = validate(repositories(None, Some(fork_of(None))), &upstream()).unwrap_err();

        assert!(
            matches!(error, SyncError::UpstreamNotFound(repository) if repository == upstream())
        );
    }

    #[test]
    fn missing_fork_reports_the_lookup_name() {
        let error = validate(
            repositories(Some(remote("microsoft", "winget-pkgs", "master")), None),
            &upstream(),
        )
        .unwrap_err();

        assert!(matches!(
            error,
            SyncError::ForkNotFound { fork, .. } if fork == Repository::new("octocat", "winget-pkgs")
        ));
    }

    #[test]
    fn repository_that_is_not_a_fork() {
        let not_a_fork = RemoteFork {
            is_fork: false,
            ..fork_of(None)
        };

        let error = validate(
            repositories(
                Some(remote("microsoft", "winget-pkgs", "master")),
                Some(not_a_fork),
            ),
            &upstream(),
        )
        .unwrap_err();

        assert!(matches!(error, SyncError::NotAFork { .. }));
    }

    #[test]
    fn fork_of_another_repository() {
        let error = validate(
            repositories(
                Some(remote("microsoft", "winget-pkgs", "master")),
                Some(fork_of(Some("contoso/winget-pkgs"))),
            ),
            &upstream(),
        )
        .unwrap_err();

        assert!(matches!(
            error,
            SyncError::ForkOfOtherRepository { parent, .. } if parent == "contoso/winget-pkgs"
        ));
    }

    #[test]
    fn fork_with_inaccessible_parent() {
        let error = validate(
            repositories(
                Some(remote("microsoft", "winget-pkgs", "master")),
                Some(fork_of(None)),
            ),
            &upstream(),
        )
        .unwrap_err();

        assert!(matches!(error, SyncError::ParentInaccessible { .. }));
    }

    #[test]
    fn branches_default_to_each_default_branch() {
        assert_eq!(
            Branches::new(None, "main", "master"),
            Branches {
                fork: "main",
                upstream: "master"
            }
        );
    }

    #[test]
    fn given_branch_is_used_on_both_sides() {
        assert_eq!(
            Branches::new(Some("release"), "main", "master"),
            Branches {
                fork: "release",
                upstream: "release"
            }
        );
    }
}
