use crate::github::graphql::get_repository_info::RepositoryVariablesFields;
use crate::github::graphql::github_schema::github_schema as schema;
use std::fmt::{Display, Formatter};
use url::Url;

#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Query", variables = "RepositoryVariables")]
pub struct GetBranches {
    #[arguments(owner: $owner, name: $name)]
    pub repository: Option<Repository>,
}

/// <https://docs.github.com/graphql/reference/objects#repository>
#[derive(cynic::QueryFragment)]
pub struct Repository {
    pub id: cynic::Id,
    pub default_branch_ref: Option<DefaultBranchRef>,
    #[arguments(first: 100, refPrefix: "refs/heads/")]
    pub refs: Option<RefConnection>,
}

/// <https://docs.github.com/graphql/reference/objects#refconnection>
#[derive(cynic::QueryFragment)]
pub struct RefConnection {
    #[cynic(flatten)]
    pub nodes: Vec<PullRequestBranchRef>,
}

/// <https://docs.github.com/graphql/reference/objects#ref>
#[derive(cynic::QueryFragment, Hash, PartialEq, Eq)]
#[cynic(graphql_type = "Ref")]
pub struct PullRequestBranchRef {
    pub name: String,
    #[arguments(first: 5)]
    pub associated_pull_requests: PullRequestConnection,
}

/// <https://docs.github.com/graphql/reference/objects#ref>
#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Ref")]
pub struct DefaultBranchRef {
    pub name: String,
}

/// <https://docs.github.com/graphql/reference/objects#pullrequestconnection>
#[derive(cynic::QueryFragment, Hash, PartialEq, Eq)]
pub struct PullRequestConnection {
    #[cynic(flatten)]
    pub nodes: Vec<PullRequest>,
}

/// <https://docs.github.com/graphql/reference/objects#pullrequest>
#[derive(cynic::QueryFragment, Hash, PartialEq, Eq)]
pub struct PullRequest {
    pub title: String,
    pub url: Url,
    pub state: PullRequestState,
    pub repository: PullRequestRepository,
}

impl Display for PullRequest {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.title)
    }
}

/// <https://docs.github.com/graphql/reference/objects#repository>
#[derive(cynic::QueryFragment, Hash, PartialEq, Eq)]
#[cynic(graphql_type = "Repository")]
pub struct PullRequestRepository {
    pub name_with_owner: String,
}

/// <https://docs.github.com/graphql/reference/enums#pullrequeststate>
#[derive(cynic::Enum, Clone, Copy, Hash, PartialEq, Eq)]
pub enum PullRequestState {
    Closed,
    Merged,
    Open,
}

impl Display for PullRequestState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Merged => "a merged",
                Self::Open => "an open",
                Self::Closed => "a closed",
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::github::github_client::{MICROSOFT, WINGET_PKGS};
    use crate::github::graphql::get_branches::GetBranches;
    use crate::github::graphql::get_repository_info::RepositoryVariables;
    use cynic::QueryBuilder;
    use indoc::indoc;

    #[test]
    fn get_branches_query_output() {
        const GET_BRANCHES_QUERY: &str = indoc! {r#"
            query GetBranches($owner: String!, $name: String!) {
              repository(owner: $owner, name: $name) {
                id
                defaultBranchRef {
                  name
                }
                refs(first: 100, refPrefix: "refs/heads/") {
                  nodes {
                    name
                    associatedPullRequests(first: 5) {
                      nodes {
                        title
                        url
                        state
                        repository {
                          nameWithOwner
                        }
                      }
                    }
                  }
                }
              }
            }
        "#};

        let operation = GetBranches::build(RepositoryVariables {
            owner: MICROSOFT,
            name: WINGET_PKGS,
        });

        assert_eq!(operation.query, GET_BRANCHES_QUERY);
    }
}
