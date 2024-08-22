use crate::github::graphql::github_schema::github_schema as schema;
use crate::github::graphql::types::GitObjectId;
use url::Url;
/*
query GetRepositoryInfo($owner: String!, $name: String!) {
  repository(owner: $owner, name: $name) {
    id
    nameWithOwner
    url
    defaultBranchRef {
      name
      id
      target {
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
*/

#[derive(cynic::QueryVariables)]
pub struct RepositoryVariables<'a> {
    pub name: &'a str,
    pub owner: &'a str,
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
