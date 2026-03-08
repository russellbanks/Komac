use std::io::Write;

use bon::Builder;
use color_eyre::eyre::eyre;
use cynic::{GraphQlResponse, MutationBuilder, http::ReqwestExt};
use owo_colors::OwoColorize;
use url::Url;

use super::{
    super::{GitHubError, client::GitHub},
    GRAPHQL_URL, github_schema as schema,
};
use crate::terminal::{Hyperlinkable, SUPPORTS_HYPERLINKS};

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
#[derive(Clone, cynic::QueryFragment)]
pub struct PullRequest {
    number: i32,
    repository: Repository,
    url: Url,
}

impl PullRequest {
    /// Returns the repository's name with the owner that the pull request is submitted to.
    #[must_use]
    #[inline]
    pub const fn name_with_owner(&self) -> &str {
        self.repository.name_with_owner.as_str()
    }

    /// Returns the HTTP URL for this pull request.
    #[must_use]
    #[inline]
    pub const fn url(&self) -> &Url {
        &self.url
    }

    pub fn print_success(&self) {
        let mut stdout = anstream::stdout();

        let _ = writeln!(
            stdout,
            "{} created {}",
            "Successfully".green(),
            format_args!(
                "{repository_name_with_owner}#{number}",
                repository_name_with_owner = self.name_with_owner(),
                number = self.number
            )
            .hyperlink(self.url())
        );

        // If the terminal doesn't support hyperlinks, print the pull request's URL on a new line
        if !*SUPPORTS_HYPERLINKS {
            let _ = writeln!(stdout, "{}", self.url());
        }
    }
}

#[derive(Clone, cynic::QueryFragment)]
pub struct Repository {
    name_with_owner: String,
}

/// <https://docs.github.com/graphql/reference/input-objects#createpullrequestinput>
#[derive(Builder, cynic::InputObject)]
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

impl GitHub {
    pub async fn create_pull_request(
        &self,
        repository_id: &cynic::Id,
        fork_id: &cynic::Id,
        fork_ref_name: &str,
        branch_name: &str,
        title: &str,
        body: &str,
    ) -> Result<PullRequest, GitHubError> {
        let operation = CreatePullRequest::build(CreatePullRequestVariables {
            input: CreatePullRequestInput::builder()
                .base_ref_name(branch_name)
                .body(body)
                .head_ref_name(fork_ref_name)
                .head_repository_id(fork_id)
                .repository_id(repository_id)
                .title(title)
                .build(),
        });

        let GraphQlResponse { data, errors } =
            self.0.post(GRAPHQL_URL).run_graphql(operation).await?;

        data.and_then(|data| data.create_pull_request?.pull_request)
            .ok_or_else(|| {
                GitHubError::graphql_errors(eyre!("failed to create pull request"), errors)
            })
    }
}

#[cfg(test)]
mod tests {
    use cynic::{Id, MutationBuilder};
    use indoc::indoc;

    use super::{CreatePullRequest, CreatePullRequestInput, CreatePullRequestVariables};

    #[test]
    fn create_commit_output() {
        const CREATE_PULL_REQUEST_MUTATION: &str = indoc! {"
            mutation CreatePullRequest($input: CreatePullRequestInput!) {
              createPullRequest(input: $input) {
                pullRequest {
                  number
                  repository {
                    nameWithOwner
                  }
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
