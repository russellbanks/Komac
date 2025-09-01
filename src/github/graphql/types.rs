use crate::github::graphql::github_schema::github_schema as schema;

/// <https://docs.github.com/graphql/reference/scalars#base64string>
#[derive(cynic::Scalar)]
pub struct Base64String(String);

impl Base64String {
    pub fn new<S: Into<String>>(base_64: S) -> Self {
        Self(base_64.into())
    }
}

impl From<String> for Base64String {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

/// <https://docs.github.com/graphql/reference/scalars#gitobjectid>
#[derive(cynic::Scalar, PartialEq, Eq, Clone)]
#[cynic(graphql_type = "GitObjectID")]
pub struct GitObjectId(String);

impl GitObjectId {
    pub fn new<S: Into<String>>(git_object_id: S) -> Self {
        Self(git_object_id.into())
    }
}

impl From<String> for GitObjectId {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

/// <https://docs.github.com/graphql/reference/scalars#gitrefname>
#[derive(cynic::Scalar)]
#[cynic(graphql_type = "GitRefname")]
pub struct GitRefName(String);

impl GitRefName {
    pub fn new<S: Into<String>>(git_ref_name: S) -> Self {
        Self(git_ref_name.into())
    }
}

impl From<String> for GitRefName {
    fn from(s: String) -> Self {
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

impl From<String> for Html {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}
