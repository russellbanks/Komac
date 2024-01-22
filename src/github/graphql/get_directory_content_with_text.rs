use crate::github::graphql::get_directory_content::GetDirectoryContentVariablesFields;
use crate::github::graphql::github_schema::github_schema as schema;

/*
query GetDirectoryContentWithText($owner: String!, $name: String!, $expression: String!) {
  repository(owner: $owner, name: $name) {
    object(expression: $expression) {
      ... on Tree {
        entries {
          name
          object {
            ... on Blob {
              text
            }
          }
        }
      }
    }
  }
}
*/

#[derive(cynic::QueryFragment)]
pub struct Tree {
    #[cynic(flatten)]
    pub entries: Vec<TreeEntry>,
}

#[derive(cynic::QueryFragment)]
pub struct TreeEntry {
    pub name: String,
    #[cynic(recurse = "1")]
    pub object: Option<GitObject>,
}

#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Query", variables = "GetDirectoryContentVariables")]
pub struct GetDirectoryContentWithText {
    #[arguments(owner: $owner, name: $name)]
    pub repository: Option<Repository>,
}

#[derive(cynic::QueryFragment)]
#[cynic(variables = "GetDirectoryContentVariables")]
pub struct Repository {
    #[arguments(expression: $expression)]
    pub object: Option<GitObject>,
}

#[derive(cynic::QueryFragment)]
pub struct Blob {
    pub text: Option<String>,
}

#[derive(cynic::InlineFragments)]
pub enum GitObject {
    Tree(Tree),
    Blob(Blob),
    #[cynic(fallback)]
    Unknown,
}
