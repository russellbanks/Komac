use crate::github::graphql::github_schema::github_schema as schema;
use url::Url;

/*
mutation CreatePullRequest($input: CreatePullRequestInput!){
  createPullRequest(input: $input) {
    pullRequest {
      url
    }
  }
}
*/

#[derive(cynic::QueryVariables)]
pub struct CreatePullRequestVariables<'a> {
    pub input: CreatePullRequestInput<'a>,
}

#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Mutation", variables = "CreatePullRequestVariables")]
pub struct CreatePullRequest {
    #[arguments(input: $input)]
    pub create_pull_request: Option<CreatePullRequestPayload>,
}

#[derive(cynic::QueryFragment)]
pub struct CreatePullRequestPayload {
    pub pull_request: Option<PullRequest>,
}

#[derive(cynic::QueryFragment)]
pub struct PullRequest {
    pub url: Url,
}

#[derive(cynic::InputObject)]
pub struct CreatePullRequestInput<'a> {
    pub base_ref_name: &'a str,
    #[cynic(skip_serializing_if = "Option::is_none")]
    pub body: Option<&'a str>,
    #[cynic(skip_serializing_if = "Option::is_none")]
    pub client_mutation_id: Option<&'a str>,
    #[cynic(skip_serializing_if = "Option::is_none")]
    pub draft: Option<bool>,
    pub head_ref_name: &'a str,
    #[cynic(skip_serializing_if = "Option::is_none")]
    pub head_repository_id: Option<&'a cynic::Id>,
    #[cynic(skip_serializing_if = "Option::is_none")]
    pub maintainer_can_modify: Option<bool>,
    pub repository_id: &'a cynic::Id,
    pub title: &'a str,
}
