mod base64_string;
mod git_ref_name;
mod pull_request;

pub use base64_string::Base64String;
pub use git_ref_name::GitRefName;
pub use pull_request::PullRequestState;

use super::github_schema as schema;

/// <https://docs.github.com/graphql/reference/scalars#gitobjectid>
#[derive(cynic::Scalar, PartialEq, Eq, Clone)]
#[cynic(graphql_type = "GitObjectID")]
pub struct GitObjectId(String);

impl GitObjectId {
    pub fn new<S: Into<String>>(git_object_id: S) -> Self {
        Self(git_object_id.into())
    }
}

impl<T> From<T> for GitObjectId
where
    T: Into<String>,
{
    fn from(s: T) -> Self {
        Self::new(s)
    }
}

/// <https://docs.github.com/en/graphql/reference/scalars#html>
#[derive(cynic::Scalar)]
#[cynic(graphql_type = "HTML")]
pub struct Html(String);

impl Html {
    pub fn new<S: Into<String>>(html: S) -> Self {
        Self(html.into())
    }

    #[inline]
    pub const fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl<T> From<T> for Html
where
    T: Into<String>,
{
    fn from(s: T) -> Self {
        Self::new(s)
    }
}
