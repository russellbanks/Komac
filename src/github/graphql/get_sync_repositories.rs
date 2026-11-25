//! Fetches upstream and a fork of it in one request, for `komac sync`.

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
    pub upstream: Option<UpstreamRepository>,
    pub viewer: Viewer,
}

/// The authenticated user, and their repository with upstream's name.
#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "User", variables = "ViewerForkVariables")]
pub struct Viewer {
    pub login: String,
    #[arguments(name: $upstream_name)]
    pub repository: Option<ForkRepository>,
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
    pub upstream: Option<UpstreamRepository>,
    #[arguments(owner: $fork_owner, name: $fork_name)]
    #[cynic(rename = "repository", alias)]
    pub fork: Option<ForkRepository>,
}

#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Repository")]
pub struct UpstreamRepository {
    pub name_with_owner: String,
    pub url: Url,
    pub default_branch_ref: Option<DefaultBranchRef>,
}

#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Repository")]
pub struct ForkRepository {
    pub name_with_owner: String,
    pub url: Url,
    pub default_branch_ref: Option<DefaultBranchRef>,
    pub is_fork: bool,
    pub parent: Option<ParentRepository>,
}

#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Ref")]
pub struct DefaultBranchRef {
    pub name: String,
}

#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Repository")]
pub struct ParentRepository {
    pub name_with_owner: String,
}

impl GitHub {
    /// Gets upstream and the authenticated user's repository with upstream's name.
    ///
    /// A repository that is not found is `None` rather than an error.
    pub async fn get_viewer_fork(
        &self,
        upstream: &Repository,
    ) -> Result<GetViewerFork, GitHubError> {
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
        })
    }

    /// Gets upstream and the repository named `fork`.
    ///
    /// A repository that is not found is `None` rather than an error.
    pub async fn get_named_fork(
        &self,
        upstream: &Repository,
        fork: &Repository,
    ) -> Result<GetNamedFork, GitHubError> {
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
        })
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::{
        GetNamedFork, GetViewerFork, NamedForkVariables, QueryBuilder, ViewerForkVariables,
    };

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
}
