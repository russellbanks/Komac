//! Fetches everything `komac sync` needs about upstream and the fork in one request.

use color_eyre::eyre::eyre;
use cynic::{GraphQlResponse, QueryBuilder, http::ReqwestExt};
use url::Url;

use super::{GRAPHQL_URL, github_schema as schema};
use crate::github::{GitHubError, Repository, client::GitHub};

/// Variables for looking up upstream and the authenticated user's fork of it.
#[derive(cynic::QueryVariables)]
pub struct ViewerForkVariables<'a> {
    pub upstream_owner: &'a str,
    pub upstream_name: &'a str,
}

/// Gets upstream and the authenticated user's repository with upstream's name.
#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Query", variables = "ViewerForkVariables")]
pub struct GetViewerFork {
    #[arguments(owner: $upstream_owner, name: $upstream_name)]
    #[cynic(rename = "repository", alias)]
    upstream: Option<UpstreamRepository>,
    viewer: Viewer,
}

#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "User", variables = "ViewerForkVariables")]
struct Viewer {
    login: String,
    #[arguments(name: $upstream_name)]
    repository: Option<ForkRepository>,
}

/// Variables for looking up upstream and a fork named by its owner and name.
#[derive(cynic::QueryVariables)]
pub struct NamedForkVariables<'a> {
    pub upstream_owner: &'a str,
    pub upstream_name: &'a str,
    pub fork_owner: &'a str,
    pub fork_name: &'a str,
}

/// Gets upstream and a fork named by its owner and name.
#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Query", variables = "NamedForkVariables")]
pub struct GetNamedFork {
    #[arguments(owner: $upstream_owner, name: $upstream_name)]
    #[cynic(rename = "repository", alias)]
    upstream: Option<UpstreamRepository>,
    #[arguments(owner: $fork_owner, name: $fork_name)]
    #[cynic(rename = "repository", alias)]
    fork: Option<ForkRepository>,
}

#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Repository")]
struct UpstreamRepository {
    name_with_owner: String,
    url: Url,
    default_branch_ref: Option<DefaultBranchRef>,
}

#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Repository")]
struct ForkRepository {
    name_with_owner: String,
    url: Url,
    default_branch_ref: Option<DefaultBranchRef>,
    is_fork: bool,
    parent: Option<ParentRepository>,
}

#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Ref")]
struct DefaultBranchRef {
    name: String,
}

#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Repository")]
struct ParentRepository {
    name_with_owner: String,
}

/// A repository as GitHub currently names it, which may differ from the name it was looked up by
/// if it was renamed.
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
    /// The `owner/name` of the repository this was forked from, if the token can see it.
    pub parent: Option<String>,
}

/// What GitHub returned for upstream and the fork. Either is `None` if it was not found.
#[derive(Debug)]
pub struct SyncRepositories {
    pub upstream: Option<RemoteRepository>,
    pub fork: Option<RemoteFork>,
    /// The name the fork was looked up by, for reporting a fork that was not found.
    pub fork_lookup: Repository,
}

impl GitHub {
    /// Gets upstream and the fork to sync with it in one request.
    ///
    /// The fork is the authenticated user's repository with upstream's name, unless `fork` names
    /// it. Repositories that are not found are `None`, so the caller can explain why.
    pub async fn get_sync_repositories(
        &self,
        upstream: &Repository,
        fork: Option<&Repository>,
    ) -> Result<SyncRepositories, GitHubError> {
        if let Some(fork) = fork {
            let GraphQlResponse { data, errors } = self
                .0
                .post(GRAPHQL_URL)
                .run_graphql(GetNamedFork::build(NamedForkVariables {
                    upstream_owner: upstream.owner(),
                    upstream_name: upstream.name(),
                    fork_owner: fork.owner(),
                    fork_name: fork.name(),
                }))
                .await?;

            data.ok_or_else(|| {
                GitHubError::graphql_errors(eyre!("failed to get {upstream} and {fork}"), errors)
            })?
            .into_sync_repositories(fork)
        } else {
            let GraphQlResponse { data, errors } = self
                .0
                .post(GRAPHQL_URL)
                .run_graphql(GetViewerFork::build(ViewerForkVariables {
                    upstream_owner: upstream.owner(),
                    upstream_name: upstream.name(),
                }))
                .await?;

            data.ok_or_else(|| {
                GitHubError::graphql_errors(
                    eyre!("failed to get {upstream} and your fork of it"),
                    errors,
                )
            })?
            .into_sync_repositories(upstream.name())
        }
    }
}

impl GetViewerFork {
    /// Converts the response, where the fork was looked up as `<viewer>/<upstream_name>`.
    pub fn into_sync_repositories(
        self,
        upstream_name: &str,
    ) -> Result<SyncRepositories, GitHubError> {
        Ok(SyncRepositories {
            upstream: self
                .upstream
                .map(UpstreamRepository::try_into)
                .transpose()?,
            fork: self
                .viewer
                .repository
                .map(ForkRepository::try_into)
                .transpose()?,
            fork_lookup: Repository::new(self.viewer.login, upstream_name.to_owned()),
        })
    }
}

impl GetNamedFork {
    /// Converts the response, where the fork was looked up as `fork`.
    pub fn into_sync_repositories(
        self,
        fork: &Repository,
    ) -> Result<SyncRepositories, GitHubError> {
        Ok(SyncRepositories {
            upstream: self
                .upstream
                .map(UpstreamRepository::try_into)
                .transpose()?,
            fork: self.fork.map(ForkRepository::try_into).transpose()?,
            fork_lookup: fork.clone(),
        })
    }
}

impl RemoteRepository {
    fn new(
        name_with_owner: &str,
        url: Url,
        default_branch_ref: Option<DefaultBranchRef>,
    ) -> Result<Self, GitHubError> {
        let repository = name_with_owner
            .parse::<Repository>()
            .map_err(|error| GitHubError::GraphQL(error.into()))?;

        let default_branch = default_branch_ref
            .ok_or_else(|| GitHubError::GraphQL(eyre!("{repository} has no default branch")))?
            .name;

        Ok(Self {
            repository,
            url,
            default_branch,
        })
    }
}

impl TryFrom<UpstreamRepository> for RemoteRepository {
    type Error = GitHubError;

    fn try_from(upstream: UpstreamRepository) -> Result<Self, Self::Error> {
        Self::new(
            &upstream.name_with_owner,
            upstream.url,
            upstream.default_branch_ref,
        )
    }
}

impl TryFrom<ForkRepository> for RemoteFork {
    type Error = GitHubError;

    fn try_from(fork: ForkRepository) -> Result<Self, Self::Error> {
        Ok(Self {
            remote: RemoteRepository::new(
                &fork.name_with_owner,
                fork.url,
                fork.default_branch_ref,
            )?,
            is_fork: fork.is_fork,
            parent: fork.parent.map(|parent| parent.name_with_owner),
        })
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::{
        GetNamedFork, GetViewerFork, NamedForkVariables, QueryBuilder, ViewerForkVariables,
    };
    use crate::github::Repository;

    #[test]
    fn get_viewer_fork_query() {
        const QUERY: &str = indoc! {r#"
            query GetViewerFork($upstreamOwner: String!, $upstreamName: String!) {
              upstream: repository(owner: $upstreamOwner, name: $upstreamName) {
                nameWithOwner
                url
                defaultBranchRef {
                  name
                }
              }
              viewer {
                login
                repository(name: $upstreamName) {
                  nameWithOwner
                  url
                  defaultBranchRef {
                    name
                  }
                  isFork
                  parent {
                    nameWithOwner
                  }
                }
              }
            }
        "#};

        let operation = GetViewerFork::build(ViewerForkVariables {
            upstream_owner: "microsoft",
            upstream_name: "winget-pkgs",
        });

        assert_eq!(operation.query, QUERY);
    }

    #[test]
    fn get_named_fork_query() {
        const QUERY: &str = indoc! {r#"
            query GetNamedFork($upstreamOwner: String!, $upstreamName: String!, $forkOwner: String!, $forkName: String!) {
              upstream: repository(owner: $upstreamOwner, name: $upstreamName) {
                nameWithOwner
                url
                defaultBranchRef {
                  name
                }
              }
              fork: repository(owner: $forkOwner, name: $forkName) {
                nameWithOwner
                url
                defaultBranchRef {
                  name
                }
                isFork
                parent {
                  nameWithOwner
                }
              }
            }
        "#};

        let operation = GetNamedFork::build(NamedForkVariables {
            upstream_owner: "microsoft",
            upstream_name: "winget-pkgs",
            fork_owner: "octocat",
            fork_name: "renamed-pkgs",
        });

        assert_eq!(operation.query, QUERY);
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
        let repositories = viewer_response(FORK)
            .into_sync_repositories("winget-pkgs")
            .unwrap();

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
        let repositories = viewer_response("null")
            .into_sync_repositories("winget-pkgs")
            .unwrap();

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

        let repositories = response
            .into_sync_repositories(&Repository::new("octocat", "missing"))
            .unwrap();

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

        assert!(
            viewer_response(&fork)
                .into_sync_repositories("winget-pkgs")
                .is_err()
        );
    }
}
