use crate::github::graphql::get_repository_info::RepositoryVariablesFields;
use crate::github::graphql::github_schema::github_schema as schema;

/*
query GetBranches($owner: String!, $name: String!) {
  repository(name: $name, owner: $owner) {
    defaultBranchRef {
      name
      id
    }
    refs(first: 100, refPrefix: "refs/heads/") {
      nodes {
        name
        id
      }
    }
  }
}
*/

#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Query", variables = "RepositoryVariables")]
pub struct GetBranches {
    #[arguments(name: $name, owner: $owner)]
    pub repository: Option<Repository>,
}

#[derive(cynic::QueryFragment)]
pub struct Repository {
    pub default_branch_ref: Option<Ref>,
    #[arguments(first: 100, refPrefix: "refs/heads/")]
    pub refs: Option<RefConnection>,
}

#[derive(cynic::QueryFragment)]
pub struct RefConnection {
    #[cynic(flatten)]
    pub nodes: Vec<Ref>,
}

#[derive(cynic::QueryFragment)]
pub struct Ref {
    pub name: String,
    pub id: cynic::Id,
}
