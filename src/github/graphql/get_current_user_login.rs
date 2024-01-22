use crate::github::graphql::github_schema::github_schema as schema;

/*
query GetCurrentUserLogin {
  viewer {
    login
  }
}
*/

#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Query")]
pub struct GetCurrentUserLogin {
    pub viewer: User,
}

#[derive(cynic::QueryFragment)]
pub struct User {
    pub login: String,
}
