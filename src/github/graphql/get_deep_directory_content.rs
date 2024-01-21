use crate::github::graphql::get_directory_content::GetDirectoryContentVariablesFields;
use crate::github::graphql::github_schema::github_schema as schema;

/*
query GetDeepDirectoryContent($owner: String!, $name: String!, $expression: String!) {
  repository(name: $name, owner: $owner) {
    object(expression: $expression) {
      ... on Tree {
        entries {
          name
          object {
            ... on Tree {
              entries {
                type
              }
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
    #[cynic(rename = "type")]
    pub type_: String,
}

#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Tree")]
pub struct Tree2 {
    #[cynic(flatten)]
    pub entries: Vec<TreeEntry2>,
}

#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "TreeEntry")]
pub struct TreeEntry2 {
    pub name: String,
    pub object: Option<DeepGitObject>,
}

#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Query", variables = "GetDirectoryContentVariables")]
pub struct GetDeepDirectoryContent {
    #[arguments(name: $name, owner: $owner)]
    pub repository: Option<Repository>,
}

#[derive(cynic::QueryFragment)]
#[cynic(variables = "GetDirectoryContentVariables")]
pub struct Repository {
    #[arguments(expression: $expression)]
    pub object: Option<GitObject2>,
}

#[derive(cynic::InlineFragments)]
#[cynic(graphql_type = "GitObject")]
pub enum GitObject2 {
    Tree2(Tree2),
    #[cynic(fallback)]
    Unknown,
}

#[derive(cynic::InlineFragments)]
#[cynic(graphql_type = "GitObject")]
pub enum DeepGitObject {
    Tree(Tree),
    #[cynic(fallback)]
    Unknown,
}
