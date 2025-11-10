use std::fmt::{Display, Formatter};

use url::Url;

use super::{github_schema as schema, types::PullRequestState};
use crate::github::graphql::get_repository_info::RepositoryOwner;

#[derive(cynic::QueryVariables)]
pub struct GetBranchesVariables<'a> {
    pub owner: &'a str,
    pub name: &'a str,
    pub cursor: Option<&'a str>,
}

#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Query", variables = "GetBranchesVariables")]
pub struct GetBranches {
    #[arguments(owner: $owner, name: $name)]
    pub repository: Option<Repository>,
}

/// <https://docs.github.com/graphql/reference/objects#repository>
#[derive(cynic::QueryFragment)]
#[cynic(variables = "GetBranchesVariables")]
pub struct Repository {
    pub id: cynic::Id,
    pub default_branch_ref: Option<DefaultBranchRef>,
    #[arguments(first: 100, after: $cursor, refPrefix: "refs/heads/")]
    pub refs: Option<RefConnection>,
}

/// <https://docs.github.com/graphql/reference/objects#refconnection>
#[derive(cynic::QueryFragment)]
pub struct RefConnection {
    #[cynic(rename = "nodes", flatten)]
    pub branches: Vec<PullRequestBranchRef>,
    pub page_info: PageInfo,
}

#[derive(cynic::QueryFragment)]
pub struct PageInfo {
    pub end_cursor: Option<String>,
    pub has_next_page: bool,
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
    #[cynic(rename = "nodes", flatten)]
    pub pull_requests: Vec<PullRequest>,
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
    pub owner: RepositoryOwner,
    pub name: String,
}

impl PullRequestRepository {
    pub const fn owner(&self) -> &str {
        self.owner.login.as_str()
    }
}

#[cfg(test)]
mod tests {
    use cynic::QueryBuilder;
    use indoc::indoc;

    use super::{GetBranches, GetBranchesVariables};
    use crate::github::{MICROSOFT, WINGET_PKGS};

    #[test]
    fn get_branches_query_output() {
        const GET_BRANCHES_QUERY: &str = indoc! {r#"
            query GetBranches($owner: String!, $name: String!, $cursor: String) {
              repository(owner: $owner, name: $name) {
                id
                defaultBranchRef {
                  name
                }
                refs(first: 100, after: $cursor, refPrefix: "refs/heads/") {
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
                  pageInfo {
                    endCursor
                    hasNextPage
                  }
                }
              }
            }
        "#};

        let operation = GetBranches::build(GetBranchesVariables {
            owner: MICROSOFT,
            name: WINGET_PKGS,
            cursor: None,
        });

        assert_eq!(operation.query, GET_BRANCHES_QUERY);
    }
}
