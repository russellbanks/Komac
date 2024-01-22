use crate::github::graphql::get_directory_content::GetDirectoryContentVariablesFields;
use crate::github::graphql::github_schema::github_schema as schema;

#[derive(cynic::QueryFragment)]
pub struct Tree {
    #[cynic(flatten)]
    pub entries: Vec<TreeEntry>,
}

#[derive(cynic::QueryFragment)]
pub struct TreeEntry {
    pub name: String,
    #[cynic(rename = "type")]
    pub type_: String,
    #[cynic(recurse = "1")]
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
    pub object: Option<DeepGitObject>,
}

#[derive(cynic::InlineFragments)]
#[cynic(graphql_type = "GitObject")]
pub enum DeepGitObject {
    Tree(Tree),
    #[cynic(fallback)]
    Unknown,
}
