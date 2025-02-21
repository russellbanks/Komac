use chrono::{DateTime, Utc};
use url::Url;

use crate::github::graphql::{
    get_branches::PullRequestState, github_schema::github_schema as schema,
};

#[derive(cynic::QueryVariables)]
pub struct GetExistingPullRequestVariables<'a> {
    pub query: &'a str,
}

#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Query", variables = "GetExistingPullRequestVariables")]
pub struct GetExistingPullRequest {
    #[arguments(first: 1, type: ISSUE, query: $query)]
    pub search: SearchResultItemConnection,
}

#[derive(cynic::QueryFragment)]
pub struct SearchResultItemConnection {
    #[cynic(flatten)]
    pub edges: Vec<SearchResultItemEdge>,
}

#[derive(cynic::QueryFragment)]
pub struct SearchResultItemEdge {
    pub node: Option<SearchResultItem>,
}

#[derive(cynic::QueryFragment)]
pub struct PullRequest {
    pub url: Url,
    pub state: PullRequestState,
    pub created_at: DateTime<Utc>,
}

#[derive(cynic::InlineFragments)]
pub enum SearchResultItem {
    PullRequest(PullRequest),
    #[cynic(fallback)]
    Unknown,
}

impl SearchResultItem {
    pub fn into_pull_request(self) -> Option<PullRequest> {
        match self {
            Self::PullRequest(pull_request) => Some(pull_request),
            Self::Unknown => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use cynic::QueryBuilder;
    use indoc::indoc;

    use crate::github::graphql::get_existing_pull_request::{
        GetExistingPullRequest, GetExistingPullRequestVariables,
    };

    #[test]
    fn get_existing_pull_request_output() {
        const GET_EXISTING_PULL_REQUEST_QUERY: &str = indoc! {r#"
            query GetExistingPullRequest($query: String!) {
              search(first: 1, type: ISSUE, query: $query) {
                edges {
                  node {
                    __typename
                    ... on PullRequest {
                      url
                      state
                      createdAt
                    }
                  }
                }
              }
            }
        "#};

        let operation =
            GetExistingPullRequest::build(GetExistingPullRequestVariables { query: "" });

        assert_eq!(operation.query, GET_EXISTING_PULL_REQUEST_QUERY);
    }
}
