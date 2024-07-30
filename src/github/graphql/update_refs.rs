use crate::github::graphql::github_schema::github_schema as schema;
use crate::github::graphql::types::{GitObjectId, GitRefName};

/*
mutation UpdateRefs($repositoryId: ID!, $refUpdates: [RefUpdate!]!) {
  updateRefs(input: {
    repositoryId: $repositoryId,
    refUpdates: $refUpdates
  }) {
    clientMutationId
  }
}
*/

#[derive(cynic::QueryVariables)]
pub struct UpdateRefsVariables<'a> {
    pub ref_updates: Vec<RefUpdate>,
    pub repository_id: &'a cynic::Id,
}

#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Mutation", variables = "UpdateRefsVariables")]
pub struct UpdateRefs {
    #[allow(dead_code)]
    #[arguments(input: { refUpdates: $ref_updates, repositoryId: $repository_id })]
    pub update_refs: Option<UpdateRefsPayload>,
}

#[derive(cynic::QueryFragment)]
pub struct UpdateRefsPayload {
    #[allow(dead_code)]
    pub client_mutation_id: Option<String>,
}

#[derive(cynic::InputObject)]
pub struct RefUpdate {
    pub after_oid: GitObjectId,
    pub before_oid: Option<GitObjectId>,
    pub force: Option<bool>,
    pub name: GitRefName,
}
