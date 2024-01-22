use crate::github::graphql::github_schema::github_schema as schema;
use std::fmt::{Display, Formatter};
use url::Url;

/*
query GetPullRequestFromBranch($owner: String!, $name: String!, $baseRefName: String!, $headRefName: String!) {
  repository(name: $name, owner: $owner) {
    pullRequests(first: 1, baseRefName: $baseRefName, headRefName: $headRefName, states: [MERGED, CLOSED]) {
      nodes {
        title
        url
        state
      }
    }
  }
}
*/

#[derive(cynic::QueryVariables)]
pub struct GetPullRequestFromBranchVariables<'a> {
    pub base_ref_name: &'a str,
    pub head_ref_name: &'a str,
    pub name: &'a str,
    pub owner: &'a str,
}

#[derive(cynic::QueryFragment)]
#[cynic(
    graphql_type = "Query",
    variables = "GetPullRequestFromBranchVariables"
)]
pub struct GetPullRequestFromBranch {
    #[arguments(name: $name, owner: $owner)]
    pub repository: Option<Repository>,
}

#[derive(cynic::QueryFragment)]
#[cynic(variables = "GetPullRequestFromBranchVariables")]
pub struct Repository {
    #[arguments(first: 1, baseRefName: $base_ref_name, headRefName: $head_ref_name, states: ["MERGED", "CLOSED"])]
    pub pull_requests: PullRequestConnection,
}

#[derive(cynic::QueryFragment)]
pub struct PullRequestConnection {
    #[cynic(flatten)]
    pub nodes: Vec<PullRequest>,
}

#[derive(cynic::QueryFragment, Hash, Eq, PartialEq)]
pub struct PullRequest {
    pub title: String,
    pub url: Url,
    pub state: PullRequestState,
}

impl Display for PullRequest {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.title)
    }
}

#[derive(cynic::Enum, Clone, Copy, Hash, Eq, PartialEq)]
pub enum PullRequestState {
    Closed,
    Merged,
    Open,
    #[cynic(fallback)]
    Other,
}
