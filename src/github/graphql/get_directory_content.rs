use crate::github::graphql::github_schema::github_schema as schema;

#[derive(cynic::QueryVariables)]
pub struct GetDirectoryContentVariables<'a> {
    pub owner: &'a str,
    pub name: &'a str,
    pub expression: &'a str,
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

impl TreeGitObject {
    pub fn into_entries(self) -> Option<Vec<TreeEntry>> {
        match self {
            Self::Tree(tree) => Some(tree.entries),
            Self::Unknown => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use cynic::QueryBuilder;
    use indoc::indoc;

    use crate::github::{
        github_client::{MICROSOFT, WINGET_PKGS},
        graphql::get_directory_content::{GetDirectoryContent, GetDirectoryContentVariables},
    };

    #[test]
    fn get_directory_content_output() {
        const GET_DIRECTORY_CONTENT_QUERY: &str = indoc! {r#"
            query GetDirectoryContent($owner: String!, $name: String!, $expression: String!) {
              repository(owner: $owner, name: $name) {
                object(expression: $expression) {
                  __typename
                  ... on Tree {
                    entries {
                      path
                    }
                  }
                }
              }
            }
        "#};

        let operation = GetDirectoryContent::build(GetDirectoryContentVariables {
            owner: MICROSOFT,
            name: WINGET_PKGS,
            expression: "",
        });

        assert_eq!(operation.query, GET_DIRECTORY_CONTENT_QUERY);
    }
}
