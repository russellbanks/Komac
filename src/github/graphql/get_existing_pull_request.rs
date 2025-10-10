use chrono::{DateTime, Utc};
use cynic::{QueryBuilder, http::ReqwestExt};
use url::Url;
use winget_types::{PackageIdentifier, PackageVersion};

use super::{
    super::{GitHubError, WINGET_PKGS_FULL_NAME, client::GitHub},
    GRAPHQL_URL, github_schema as schema,
    types::PullRequestState,
};

#[derive(cynic::QueryVariables)]
pub struct GetExistingPullRequestVariables<'a> {
    pub query: &'a str,
}

#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Query", variables = "GetExistingPullRequestVariables")]
pub struct GetExistingPullRequest {
    #[arguments(first: 100, type: ISSUE, query: $query)]
    pub search: SearchResultItemConnection,
}

impl GetExistingPullRequest {
    pub fn into_pull_requests(self) -> impl Iterator<Item = PullRequest> {
        self.search
            .nodes
            .into_iter()
            .filter_map(SearchResultItem::into_pull_request)
    }
}

#[derive(cynic::QueryFragment)]
pub struct SearchResultItemConnection {
    #[cynic(flatten)]
    pub nodes: Vec<SearchResultItem>,
}

#[derive(cynic::QueryFragment)]
pub struct PullRequest {
    pub title: String,
    pub url: Url,
    pub state: PullRequestState,
    pub created_at: DateTime<Utc>,
}

impl PullRequest {
    /// Returns `true` if the pull request has been closed without being merged.
    #[expect(unused)]
    #[inline]
    pub const fn is_closed(&self) -> bool {
        self.state.is_closed()
    }

    /// Returns `true` if the pull request has been closed by being merged.
    #[expect(unused)]
    #[inline]
    pub const fn is_merged(&self) -> bool {
        self.state.is_merged()
    }

    /// Returns `true` if the pull request is still open.
    #[inline]
    pub const fn is_open(&self) -> bool {
        self.state.is_open()
    }
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

impl GitHub {
    pub async fn get_existing_pull_request(
        &self,
        identifier: &PackageIdentifier,
        version: &PackageVersion,
    ) -> Result<Option<PullRequest>, GitHubError> {
        self
            .0
            .post(GRAPHQL_URL)
            .run_graphql(GetExistingPullRequest::build(GetExistingPullRequestVariables {
                query: &format!("repo:{WINGET_PKGS_FULL_NAME} is:pull-request in:title {identifier} {version}"),
            }))
            .await
            .map(|response| {
                response
                    .data?
                    .into_pull_requests()
                    .find(|pull_request| {
                        let title = &*pull_request.title;
                        // Check that the identifier and version are used in their entirety and not
                        // part of another package identifier or version. For example, ensuring we
                        // match against "Microsoft.Excel" not "Microsoft.Excel.Beta", or "1.2.3"
                        // and not "1.2.3-beta" as `in:title` in the query only does a 'contains'
                        // rather than a word boundary match.
                        [identifier.as_str(), version.as_str()]
                            .into_iter()
                            .all(|needle| {
                                title.match_indices(needle).any(|(index, matched)| {
                                    let before = title[..index].chars().next_back();
                                    let after = title[index + matched.len()..].chars().next();
                                    // Check whether the characters before and after the identifier
                                    // are either None (at the boundary of the title) or whitespace
                                    before.is_none_or(char::is_whitespace)
                                        && after.is_none_or(char::is_whitespace)
                                })
                            })
                    })
            })
            .map_err(GitHubError::CynicRequest)
    }
}

#[cfg(test)]
mod tests {
    use cynic::QueryBuilder;
    use indoc::indoc;

    use super::{GetExistingPullRequest, GetExistingPullRequestVariables};

    #[test]
    fn get_existing_pull_request_output() {
        const GET_EXISTING_PULL_REQUEST_QUERY: &str = indoc! {r#"
            query GetExistingPullRequest($query: String!) {
              search(first: 100, type: ISSUE, query: $query) {
                nodes {
                  __typename
                  ... on PullRequest {
                    title
                    url
                    state
                    createdAt
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
