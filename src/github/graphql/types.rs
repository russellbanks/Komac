use crate::github::graphql::github_schema::github_schema as schema;
use derive_more::Deref;
use derive_new::new;

/// <https://docs.github.com/graphql/reference/scalars#base64string>
#[derive(cynic::Scalar, new)]
pub struct Base64String(#[new(into)] String);

/// <https://docs.github.com/graphql/reference/scalars#gitobjectid>
#[derive(cynic::Scalar, PartialEq, Eq, Clone, new)]
#[cynic(graphql_type = "GitObjectID")]
pub struct GitObjectId(#[new(into)] String);

/// <https://docs.github.com/graphql/reference/scalars#gitrefname>
#[derive(cynic::Scalar, new)]
#[cynic(graphql_type = "GitRefname")]
pub struct GitRefName(#[new(into)] String);

/// <https://docs.github.com/en/graphql/reference/scalars#html>
#[derive(cynic::Scalar, Deref)]
#[cynic(graphql_type = "HTML")]
pub struct Html(String);
