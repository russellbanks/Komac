use crate::github::graphql::github_schema::github_schema as schema;

/*
query GetDirectoryContent($owner: String!, $name: String!, $expression: String!) {
  repository(owner: $owner, name: $name) {
    object(expression: $expression) {
      ... on Tree {
        entries {
          path
        }
      }
    }
  }
}
*/

#[derive(cynic::QueryVariables)]
pub struct GetDirectoryContentVariables<'a> {
    pub expression: &'a str,
    pub name: &'a str,
    pub owner: &'a str,
}

#[derive(cynic::QueryFragment)]
pub struct Tree {
    #[cynic(flatten)]
    pub entries: Vec<TreeEntry>,
}

#[derive(cynic::QueryFragment)]
pub struct TreeEntry {
    pub path: Option<String>,
}

#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Query", variables = "GetDirectoryContentVariables")]
pub struct GetDirectoryContent {
    #[arguments(owner: $owner, name: $name)]
    pub repository: Option<Repository>,
}

#[derive(cynic::QueryFragment)]
#[cynic(variables = "GetDirectoryContentVariables")]
pub struct Repository {
    #[arguments(expression: $expression)]
    pub object: Option<TreeGitObject>,
}

#[derive(cynic::InlineFragments)]
#[cynic(graphql_type = "GitObject")]
pub enum TreeGitObject {
    Tree(Tree),
    #[cynic(fallback)]
    Unknown,
}
