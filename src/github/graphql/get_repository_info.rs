use crate::github::graphql::github_schema::github_schema as schema;
use crate::github::graphql::types::GitObjectId;
use url::Url;

#[derive(cynic::QueryVariables)]
pub struct RepositoryVariables<'a> {
    pub owner: &'a str,
    pub name: &'a str,
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
    use crate::github::github_client::{MICROSOFT, WINGET_PKGS};
    use crate::github::graphql::get_repository_info::{GetRepositoryInfo, RepositoryVariables};
    use cynic::QueryBuilder;
    use indoc::indoc;

    #[test]
    fn get_repository_info_output() {
        const GET_REPOSITORY_INFO_QUERY: &str = indoc! {r#"
            query GetRepositoryInfo($owner: String!, $name: String!) {
              repository(owner: $owner, name: $name) {
                id
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
