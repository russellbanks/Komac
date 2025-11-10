use url::Url;

use super::{github_schema as schema, types::GitObjectId};

#[derive(cynic::QueryVariables)]
pub struct RepositoryVariables<'a> {
    pub owner: &'a str,
    pub name: &'a str,
}

impl<'a> RepositoryVariables<'a> {
    #[inline]
    pub const fn new(owner: &'a str, name: &'a str) -> Self {
        RepositoryVariables { owner, name }
    }
}

#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Query", variables = "RepositoryVariables")]
pub struct GetRepositoryInfo {
    #[arguments(owner: $owner, name: $name)]
    pub repository: Option<Repository>,
}

#[derive(cynic::QueryFragment)]
pub struct Repository {
    pub id: cynic::Id,
    pub owner: RepositoryOwner,
    pub name_with_owner: String,
    pub url: Url,
    pub default_branch_ref: Option<Ref>,
}

#[derive(cynic::QueryFragment)]
pub struct Ref {
    pub name: String,
    pub id: cynic::Id,
    pub target: Option<TargetGitObject>,
}

#[derive(cynic::QueryFragment, Hash, PartialEq, Eq)]
pub struct RepositoryOwner {
    pub login: String,
}

#[derive(cynic::QueryFragment)]
pub struct Commit {
    pub oid: GitObjectId,
    pub history: CommitHistoryConnection,
}

#[derive(cynic::QueryFragment)]
pub struct CommitHistoryConnection {
    pub total_count: i32,
}

#[derive(cynic::InlineFragments)]
#[cynic(graphql_type = "GitObject")]
pub enum TargetGitObject {
    Commit(Commit),
    #[cynic(fallback)]
    Unknown,
}

impl TargetGitObject {
    pub fn into_commit(self) -> Option<Commit> {
        match self {
            Self::Commit(commit) => Some(commit),
            Self::Unknown => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use cynic::QueryBuilder;
    use indoc::indoc;

    use crate::github::{
        MICROSOFT, WINGET_PKGS,
        graphql::get_repository_info::{GetRepositoryInfo, RepositoryVariables},
    };

    #[test]
    fn get_repository_info_output() {
        const GET_REPOSITORY_INFO_QUERY: &str = indoc! {r#"
            query GetRepositoryInfo($owner: String!, $name: String!) {
              repository(owner: $owner, name: $name) {
                id
                owner {
                  login
                }
                nameWithOwner
                url
                defaultBranchRef {
                  name
                  id
                  target {
                    __typename
                    ... on Commit {
                      oid
                      history {
                        totalCount
                      }
                    }
                  }
                }
              }
            }
        "#};

        let operation = GetRepositoryInfo::build(RepositoryVariables {
            owner: MICROSOFT,
            name: WINGET_PKGS,
        });

        assert_eq!(operation.query, GET_REPOSITORY_INFO_QUERY);
    }
}
