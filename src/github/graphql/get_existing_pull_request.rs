use crate::github::graphql::get_pull_request_from_branch::PullRequestState;
use crate::github::graphql::github_schema::github_schema as schema;
use chrono::{DateTime, Utc};
use url::Url;

#[derive(cynic::QueryVariables)]
pub struct GetExistingPullRequestVariables<'a> {
    pub query: &'a str,
}

#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Query", variables = "GetExistingPullRequestVariables")]
pub struct GetExistingPullRequest {
    #[arguments(first: 1, type: "ISSUE", query: $query)]
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
