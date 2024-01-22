use crate::github::graphql::get_repository_info::GitObjectId;
use crate::github::graphql::github_schema::github_schema as schema;

/*
mutation CreateRef($repositoryId: ID!, $name: String!, $oid: GitObjectID!) {
  createRef(input: {
    repositoryId: $repositoryId,
    name: $name,
    oid: $oid
  }) {
    ref {
      id
      name
      target {
        oid
      }
    }
  }
}
*/

#[derive(cynic::QueryVariables)]
pub struct CreateRefVariables<'a> {
    pub name: &'a str,
    pub oid: GitObjectId,
    pub repository_id: &'a cynic::Id,
}

#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Mutation", variables = "CreateRefVariables")]
pub struct CreateRef {
    #[arguments(input: { name: $name, oid: $oid, repositoryId: $repository_id })]
    pub create_ref: Option<CreateRefPayload>,
}

#[derive(cynic::QueryFragment)]
pub struct CreateRefPayload {
    #[cynic(rename = "ref")]
    pub ref_: Option<Ref>,
}

#[derive(cynic::QueryFragment)]
pub struct Ref {
    pub id: cynic::Id,
    pub name: String,
    pub target: Option<GitObject>,
}

#[derive(cynic::QueryFragment)]
pub struct GitObject {
    pub oid: GitObjectId,
}
