use crate::github::graphql::github_schema::github_schema as schema;

#[derive(cynic::Scalar)]
pub struct Base64String(pub String);

#[derive(cynic::Scalar, PartialEq, Eq)]
#[cynic(graphql_type = "GitObjectID")]
pub struct GitObjectId(pub String);

#[derive(cynic::Scalar)]
#[cynic(graphql_type = "GitRefname")]
pub struct GitRefName(pub String);
