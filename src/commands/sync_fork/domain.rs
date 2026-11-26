//! The sync core's own types and rules. Nothing here knows how GitHub is reached or how results
//! are shown; adapters translate to and from these types.

use std::{error::Error, fmt};

use thiserror::Error;
use url::Url;

use super::summary::Commits;
use crate::{github::Repository, i18n::fl};

/// An error from a [`ForkSyncApi`](super::service::ForkSyncApi) adapter, kept opaque so the core
/// does not depend on any adapter's error type.
pub type ApiError = Box<dyn Error + Send + Sync>;

/// How the fork's branch relates to upstream's branch.
///
/// Counts are from the fork's point of view: `behind` is the number of upstream commits the fork
/// lacks, and `ahead` is the number of fork commits that upstream lacks.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Divergence {
    /// The fork and upstream point at the same commit.
    Identical,
    /// Upstream has commits the fork lacks, and the fork has nothing extra.
    Behind { by: u32 },
    /// The fork has commits upstream lacks, and upstream has nothing new.
    Ahead { by: u32 },
    /// Both sides have commits the other lacks.
    Diverged { behind: u32, ahead: u32 },
}

/// A repository as currently named, which may differ from the name it was looked up by if it was
/// renamed.
#[derive(Debug)]
pub struct RemoteRepository {
    pub repository: Repository,
    pub url: Url,
    pub default_branch: String,
}

/// The repository found where the fork was expected, and what it is a fork of.
#[derive(Debug)]
pub struct RemoteFork {
    pub remote: RemoteRepository,
    pub is_fork: bool,
    /// The `owner/name` of the repository this was forked from, if it can be seen.
    pub parent: Option<String>,
}

/// Upstream and the fork as found. Either is `None` if it was not found.
#[derive(Debug)]
pub struct SyncRepositories {
    pub upstream: Option<RemoteRepository>,
    pub fork: Option<RemoteFork>,
    /// The name the fork was looked up by, for reporting a fork that was not found.
    pub fork_lookup: Repository,
}

/// How upstream's commits were brought into the fork.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MergeKind {
    /// A merge commit was created, because the fork had diverged.
    Merge,
    /// The fork's branch was moved forward to upstream's.
    FastForward,
}

impl fmt::Display for MergeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Merge => "merge",
            Self::FastForward => "fast-forward",
        })
    }
}

/// What happened when upstream was merged into the fork.
#[derive(Debug, PartialEq, Eq)]
pub enum MergeResult {
    Merged(MergeKind),
    /// Nothing was merged. The message explains why.
    NothingMerged {
        message: String,
    },
}

/// What a sync found or did.
#[derive(Debug, PartialEq, Eq)]
pub enum SyncOutcome {
    /// The fork already matches upstream.
    UpToDate,
    /// The fork has commits upstream lacks, and upstream has nothing new to merge.
    ForkAhead(Commits),
    /// Upstream's commits were merged into the fork.
    Merged { commits: Commits, kind: MergeKind },
    /// Nothing was merged even though the comparison found upstream commits, e.g. because the
    /// fork was synced in between. The message explains why.
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
    #[error("{}", fl!("error-sync-upstream-not-found", upstream = .0.to_string()))]
    UpstreamNotFound(Repository),
    #[error("{}", fl!("error-sync-fork-not-found", fork = .fork.to_string()))]
    ForkNotFound {
        fork: Repository,
        upstream: Repository,
    },
    #[error("{}", fl!(
        "error-sync-not-a-fork",
        fork = .fork.to_string(),
        upstream = .upstream.to_string()
    ))]
    NotAFork {
        fork: Repository,
        upstream: Repository,
    },
    #[error("{}", fl!(
        "error-sync-fork-of-other",
        fork = .fork.to_string(),
        parent = .parent.as_str(),
        upstream = .upstream.to_string()
    ))]
    ForkOfOtherRepository {
        fork: Repository,
        parent: String,
        upstream: Repository,
    },
    #[error("{}", fl!(
        "error-sync-parent-inaccessible",
        fork = .fork.to_string(),
        upstream = .upstream.to_string()
    ))]
    ParentInaccessible {
        fork: Repository,
        upstream: Repository,
    },
    #[error("{}", fl!("error-sync-merge", summary = .summary.as_str()))]
    Merge { summary: String, source: ApiError },
    #[error(transparent)]
    Api(ApiError),
}

/// Upstream and a fork of it that can be synced.
#[derive(Debug)]
pub struct Target {
    pub upstream: RemoteRepository,
    pub fork: RemoteRepository,
}

/// Checks that both repositories exist and that the fork's parent is upstream, since GitHub syncs
/// a fork only from its direct parent.
pub fn validate(
    repositories: SyncRepositories,
    upstream: &Repository,
) -> Result<Target, SyncError> {
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
pub struct Branches<'a> {
    pub fork: &'a str,
    pub upstream: &'a str,
}

impl<'a> Branches<'a> {
    /// Uses each repository's default branch, unless a branch was given.
    ///
    /// A given branch is compared with upstream's branch of the same name, matching
    /// `gh repo sync --branch`. GitHub's API docs don't say which upstream branch merge-upstream
    /// uses for a non-default branch.
    pub fn new(branch: Option<&'a str>, fork_default: &'a str, upstream_default: &'a str) -> Self {
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
pub(super) mod tests {
    use rstest::rstest;

    use super::{Branches, RemoteFork, RemoteRepository, SyncError, SyncRepositories, validate};
    use crate::github::Repository;

    pub fn upstream() -> Repository {
        Repository::new("microsoft", "winget-pkgs")
    }

    pub fn remote(owner: &str, name: &str, default_branch: &str) -> RemoteRepository {
        RemoteRepository {
            repository: Repository::new(owner.to_owned(), name.to_owned()),
            url: format!("https://github.com/{owner}/{name}")
                .parse()
                .unwrap(),
            default_branch: default_branch.to_owned(),
        }
    }

    pub fn fork_of(parent: Option<&str>) -> RemoteFork {
        RemoteFork {
            remote: remote("octocat", "winget-pkgs", "main"),
            is_fork: true,
            parent: parent.map(str::to_owned),
        }
    }

    pub fn repositories(
        upstream: Option<RemoteRepository>,
        fork: Option<RemoteFork>,
    ) -> SyncRepositories {
        SyncRepositories {
            upstream,
            fork,
            fork_lookup: Repository::new("octocat", "winget-pkgs"),
        }
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
            SyncError::ForkNotFound { ref fork, .. } if *fork == Repository::new("octocat", "winget-pkgs")
        ));
    }

    /// Hints about command-line flags belong to the CLI, which adds them as a suggestion.
    #[test]
    fn missing_fork_message_does_not_name_cli_flags() {
        let error = validate(
            repositories(Some(remote("microsoft", "winget-pkgs", "master")), None),
            &upstream(),
        )
        .unwrap_err();

        assert_eq!(error.to_string(), "octocat/winget-pkgs was not found");
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
