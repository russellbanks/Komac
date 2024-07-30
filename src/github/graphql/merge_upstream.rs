use crate::github::graphql::github_schema::github_schema as schema;
use crate::github::graphql::types::GitObjectId;

/*
mutation MergeUpstream($branchRefId: ID!, $upstreamTargetOid: GitObjectID!) {
  updateRef(input: {
    refId: $branchRefId,
    oid: $upstreamTargetOid,
  }) {
    clientMutationId
  }
}
*/

#[derive(cynic::QueryVariables)]
pub struct MergeUpstreamVariables<'a> {
    pub branch_ref_id: &'a cynic::Id,
    pub upstream_target_oid: GitObjectId,
    pub force: bool,
}

#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Mutation", variables = "MergeUpstreamVariables")]
pub struct MergeUpstream {
    #[allow(dead_code)]
    #[arguments(input: { oid: $upstream_target_oid, refId: $branch_ref_id })]
    pub update_ref: Option<UpdateRefPayload>,
}

#[derive(cynic::QueryFragment)]
pub struct UpdateRefPayload {
    #[allow(dead_code)]
    pub client_mutation_id: Option<String>,
}
