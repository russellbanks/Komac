use crate::github::graphql::github_schema::github_schema as schema;

/*
mutation DeleteRef($ref: ID!) {
  deleteRef(input: {refId: $ref}) {
    clientMutationId
  }
}
*/

#[derive(cynic::QueryVariables)]
pub struct DeleteRefVariables<'a> {
    #[cynic(rename = "ref")]
    pub ref_: &'a cynic::Id,
}

#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Mutation", variables = "DeleteRefVariables")]
pub struct DeleteRef {
    #[arguments(input: { refId: $ref_ })]
    pub delete_ref: Option<DeleteRefPayload>,
}

#[derive(cynic::QueryFragment)]
pub struct DeleteRefPayload {
    pub client_mutation_id: Option<String>,
}
