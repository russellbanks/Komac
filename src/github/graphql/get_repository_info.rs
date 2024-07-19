use crate::github::graphql::github_schema::github_schema as schema;
use crate::github::graphql::types::GitObjectId;
use url::Url;
/*
query GetRepositoryInfo($owner: String!, $name: String!) {
  repository(owner: $owner, name: $name) {
    id
    defaultBranchRef {
      name
      target {
        oid
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
    pub target: Option<GitObject>,
}

#[derive(cynic::QueryFragment)]
pub struct GitObject {
    pub oid: GitObjectId,
}
