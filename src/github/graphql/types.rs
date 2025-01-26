use crate::github::graphql::github_schema::github_schema as schema;
use derive_more::Deref;

/// <https://docs.github.com/graphql/reference/scalars#base64string>
#[derive(cynic::Scalar)]
pub struct Base64String(String);

impl Base64String {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

/// <https://docs.github.com/graphql/reference/scalars#gitobjectid>
#[derive(cynic::Scalar, PartialEq, Eq, Clone)]
#[cynic(graphql_type = "GitObjectID")]
pub struct GitObjectId(String);

impl GitObjectId {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

/// <https://docs.github.com/graphql/reference/scalars#gitrefname>
#[derive(cynic::Scalar)]
#[cynic(graphql_type = "GitRefname")]
pub struct GitRefName(String);

impl GitRefName {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

/// <https://docs.github.com/en/graphql/reference/scalars#html>
#[derive(cynic::Scalar, Deref)]
#[cynic(graphql_type = "HTML")]
pub struct Html(String);
