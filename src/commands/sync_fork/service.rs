//! The sync use case and the ports it drives. It resolves and validates the fork, compares it with
//! upstream, and merges only when upstream has commits the fork lacks. It reaches GitHub only
//! through [`ForkSyncApi`] and reports progress only through [`SyncObserver`], so it runs the same
//! against GitHub, the VHS demo, and test fakes.

use std::error::Error;

use super::{
    domain::{
        Branches, Divergence, MergeResult, SyncError, SyncOutcome, SyncReport, SyncRepositories,
        Target, validate,
    },
    summary::{Commits, MergeSummary},
};
use crate::github::Repository;

/// The repository host operations a sync needs (driven port).
pub trait ForkSyncApi {
    type Error: Error + Send + Sync + 'static;

    /// Gets upstream and the fork, which is the user's repository with upstream's name unless
    /// `fork` names it. A repository that is not found is `None`.
    async fn get_sync_repositories(
        &self,
        upstream: &Repository,
        fork: Option<&Repository>,
    ) -> Result<SyncRepositories, Self::Error>;

    /// Describes how `fork_branch` in `fork` relates to `upstream_branch` in `upstream`.
    async fn compare_fork(
        &self,
        fork: &Repository,
        fork_branch: &str,
        upstream: &Repository,
        upstream_branch: &str,
    ) -> Result<Divergence, Self::Error>;

    /// Merges upstream into `branch` of `fork`.
    async fn sync_fork(&self, fork: &Repository, branch: &str) -> Result<MergeResult, Self::Error>;
}

/// Hears about a sync as it happens, e.g. to show it in a terminal (driven port).
pub trait SyncObserver {
    /// The fork has commits upstream lacks, so syncing will create a merge commit.
    fn diverged(&self, fork: &Repository, upstream: &Repository, ahead: Commits);

    /// Upstream is being merged into the fork, until the returned guard is dropped.
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

/// Syncs the fork with upstream, merging only when upstream has commits the fork lacks.
pub async fn sync<A: ForkSyncApi>(
    api: &A,
    request: &SyncRequest<'_>,
    observer: &impl SyncObserver,
) -> Result<SyncReport, SyncError> {
    let repositories = api
        .get_sync_repositories(request.upstream, request.fork.as_ref())
        .await
        .map_err(api_error)?;

    let Target { upstream, fork } = validate(repositories, request.upstream)?;

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
        .await
        .map_err(api_error)?;

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
            observer.diverged(&fork.repository, &upstream.repository, Commits(ahead));
            Commits(behind)
        }
    };

    let summary = MergeSummary {
        commits,
        upstream: &upstream.repository,
        fork: &fork.repository,
    };

    let result = {
        let _merging = observer.merging(&summary);
        api.sync_fork(&fork.repository, branches.fork)
            .await
            .map_err(|source| SyncError::Merge {
                summary: summary.to_string(),
                source: Box::new(source),
            })?
    };

    let outcome = match result {
        MergeResult::Merged(kind) => SyncOutcome::Merged { commits, kind },
        MergeResult::NothingMerged { message } => SyncOutcome::NothingMerged { message },
    };

    Ok(SyncReport {
        upstream,
        fork,
        outcome,
    })
}

fn api_error(error: impl Error + Send + Sync + 'static) -> SyncError {
    SyncError::Api(Box::new(error))
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use thiserror::Error;

    use super::{
        Commits, Divergence, ForkSyncApi, MergeResult, MergeSummary, SyncError, SyncObserver,
        SyncOutcome, SyncRepositories, SyncRequest, sync,
    };
    use crate::{
        commands::sync_fork::domain::{
            MergeKind,
            tests::{fork_of, remote, repositories, upstream},
        },
        github::Repository,
    };

    /// A fork of upstream whose default branch is `main`, while upstream's is `master`.
    fn valid_repositories() -> SyncRepositories {
        repositories(
            Some(remote("microsoft", "winget-pkgs", "master")),
            Some(fork_of(Some("microsoft/winget-pkgs"))),
        )
    }

    #[derive(Debug, Error)]
    #[error("{0}")]
    struct FakeError(&'static str);

    /// A scripted host that records the branches it is asked to compare and merge.
    struct FakeApi {
        repositories: RefCell<Option<SyncRepositories>>,
        divergence: Divergence,
        merge: Result<MergeResult, &'static str>,
        compared: RefCell<Option<(String, String)>>,
        merged: RefCell<Option<String>>,
    }

    impl FakeApi {
        fn new(divergence: Divergence, merge: Result<MergeResult, &'static str>) -> Self {
            Self::with_repositories(valid_repositories(), divergence, merge)
        }

        fn with_repositories(
            repositories: SyncRepositories,
            divergence: Divergence,
            merge: Result<MergeResult, &'static str>,
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
        type Error = FakeError;

        async fn get_sync_repositories(
            &self,
            _upstream: &Repository,
            _fork: Option<&Repository>,
        ) -> Result<SyncRepositories, FakeError> {
            Ok(self.repositories.borrow_mut().take().unwrap())
        }

        async fn compare_fork(
            &self,
            _fork: &Repository,
            fork_branch: &str,
            _upstream: &Repository,
            upstream_branch: &str,
        ) -> Result<Divergence, FakeError> {
            *self.compared.borrow_mut() =
                Some((fork_branch.to_owned(), upstream_branch.to_owned()));
            Ok(self.divergence)
        }

        async fn sync_fork(
            &self,
            _fork: &Repository,
            branch: &str,
        ) -> Result<MergeResult, FakeError> {
            *self.merged.borrow_mut() = Some(branch.to_owned());
            match &self.merge {
                Ok(MergeResult::Merged(kind)) => Ok(MergeResult::Merged(*kind)),
                Ok(MergeResult::NothingMerged { message }) => Ok(MergeResult::NothingMerged {
                    message: message.clone(),
                }),
                Err(error) => Err(FakeError(error)),
            }
        }
    }

    /// Records what it is told about the sync.
    #[derive(Default)]
    struct RecordingObserver {
        diverged: RefCell<Vec<String>>,
        merging: RefCell<Vec<String>>,
    }

    impl SyncObserver for RecordingObserver {
        fn diverged(&self, fork: &Repository, upstream: &Repository, ahead: Commits) {
            self.diverged
                .borrow_mut()
                .push(format!("{fork} has {ahead} not in {upstream}"));
        }

        fn merging(&self, summary: &MergeSummary<&Repository, &Repository>) -> impl Sized {
            self.merging.borrow_mut().push(summary.to_string());
        }
    }

    fn request<'a>(upstream: &'a Repository, branch: Option<&'a str>) -> SyncRequest<'a> {
        SyncRequest {
            upstream,
            fork: None,
            branch,
        }
    }

    fn fast_forward() -> Result<MergeResult, &'static str> {
        Ok(MergeResult::Merged(MergeKind::FastForward))
    }

    #[tokio::test]
    async fn up_to_date_fork_is_not_merged() {
        let api = FakeApi::new(Divergence::Identical, fast_forward());
        let observer = RecordingObserver::default();

        let report = sync(&api, &request(&upstream(), None), &observer)
            .await
            .unwrap();

        assert_eq!(report.outcome, SyncOutcome::UpToDate);
        assert!(api.merged.borrow().is_none());
        assert!(observer.merging.borrow().is_empty());
    }

    #[tokio::test]
    async fn fork_ahead_is_not_merged() {
        let api = FakeApi::new(Divergence::Ahead { by: 3 }, fast_forward());

        let report = sync(
            &api,
            &request(&upstream(), None),
            &RecordingObserver::default(),
        )
        .await
        .unwrap();

        assert_eq!(report.outcome, SyncOutcome::ForkAhead(Commits(3)));
        assert!(api.merged.borrow().is_none());
    }

    #[tokio::test]
    async fn behind_fork_merges_its_default_branch() {
        let api = FakeApi::new(Divergence::Behind { by: 5 }, fast_forward());
        let observer = RecordingObserver::default();

        let report = sync(&api, &request(&upstream(), None), &observer)
            .await
            .unwrap();

        assert_eq!(
            report.outcome,
            SyncOutcome::Merged {
                commits: Commits(5),
                kind: MergeKind::FastForward
            }
        );
        assert_eq!(
            *api.compared.borrow(),
            Some(("main".to_owned(), "master".to_owned()))
        );
        assert_eq!(api.merged.borrow().as_deref(), Some("main"));
        assert_eq!(
            *observer.merging.borrow(),
            ["5 commits from microsoft/winget-pkgs into octocat/winget-pkgs"]
        );
        assert!(observer.diverged.borrow().is_empty());
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
            Ok(MergeResult::Merged(MergeKind::Merge)),
        );

        let report = sync(
            &api,
            &request(&upstream(), None),
            &RecordingObserver::default(),
        )
        .await
        .unwrap();

        assert_eq!(
            report.outcome,
            SyncOutcome::Merged {
                commits: Commits(4),
                kind: MergeKind::Merge
            }
        );
    }

    #[tokio::test]
    async fn diverged_fork_is_reported_to_the_observer() {
        let api = FakeApi::new(
            Divergence::Diverged {
                behind: 4,
                ahead: 2,
            },
            Ok(MergeResult::Merged(MergeKind::Merge)),
        );
        let observer = RecordingObserver::default();

        sync(&api, &request(&upstream(), None), &observer)
            .await
            .unwrap();

        assert_eq!(
            *observer.diverged.borrow(),
            ["octocat/winget-pkgs has 2 commits not in microsoft/winget-pkgs"]
        );
    }

    #[tokio::test]
    async fn nothing_merged_keeps_the_explanation() {
        let api = FakeApi::new(
            Divergence::Behind { by: 1 },
            Ok(MergeResult::NothingMerged {
                message: "GitHub's explanation".to_owned(),
            }),
        );

        let report = sync(
            &api,
            &request(&upstream(), None),
            &RecordingObserver::default(),
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
        let api = FakeApi::new(Divergence::Behind { by: 1 }, fast_forward());

        sync(
            &api,
            &request(&upstream(), Some("release")),
            &RecordingObserver::default(),
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
            fast_forward(),
        );

        let error = sync(
            &api,
            &request(&upstream(), None),
            &RecordingObserver::default(),
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
            &RecordingObserver::default(),
        )
        .await
        .unwrap_err();

        assert_eq!(
            error.to_string(),
            "Failed to merge 5 commits from microsoft/winget-pkgs into octocat/winget-pkgs"
        );
        assert_eq!(
            std::error::Error::source(&error).map(ToString::to_string),
            Some("merge conflict".to_owned())
        );
    }
}
