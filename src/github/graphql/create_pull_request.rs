use url::Url;

use crate::github::graphql::github_schema::github_schema as schema;

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

/// <https://docs.github.com/graphql/reference/mutations#createpullrequest>
#[derive(cynic::QueryFragment)]
pub struct CreatePullRequestPayload {
    pub pull_request: Option<PullRequest>,
}

/// <https://docs.github.com/graphql/reference/objects#pullrequest>
#[derive(cynic::QueryFragment)]
pub struct PullRequest {
    pub url: Url,
}

/// <https://docs.github.com/graphql/reference/input-objects#createpullrequestinput>
#[derive(cynic::InputObject)]
pub struct CreatePullRequestInput<'a> {
    pub base_ref_name: &'a str,
    #[cynic(skip_serializing_if = "Option::is_none")]
    pub body: Option<&'a str>,
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

#[cfg(test)]
mod tests {
    use cynic::{Id, MutationBuilder};
    use indoc::indoc;

    use crate::github::graphql::create_pull_request::{
        CreatePullRequest, CreatePullRequestInput, CreatePullRequestVariables,
    };

    #[test]
    fn create_commit_output() {
        const CREATE_PULL_REQUEST_MUTATION: &str = indoc! {"
            mutation CreatePullRequest($input: CreatePullRequestInput!) {
              createPullRequest(input: $input) {
                pullRequest {
                  url
                }
              }
            }
        "};

        let id = Id::new("");
        let operation = CreatePullRequest::build(CreatePullRequestVariables {
            input: CreatePullRequestInput {
                base_ref_name: "",
                body: None,
                draft: None,
                head_ref_name: "",
                head_repository_id: None,
                maintainer_can_modify: None,
                repository_id: &id,
                title: "",
            },
        });

        assert_eq!(operation.query, CREATE_PULL_REQUEST_MUTATION);
    }
}
