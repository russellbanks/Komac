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
    pub object: Option<BlobObject>,
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
    pub object: Option<TreeObject>,
}

#[derive(cynic::QueryFragment)]
pub struct Blob {
    pub text: Option<String>,
}

#[derive(cynic::InlineFragments)]
#[cynic(graphql_type = "GitObject")]
pub enum BlobObject {
    Blob(Blob),
    #[cynic(fallback)]
    Unknown,
}

impl BlobObject {
    pub fn into_blob_text(self) -> Option<String> {
        match self {
            Self::Blob(blob) => blob.text,
            Self::Unknown => None,
        }
    }
}

#[derive(cynic::InlineFragments)]
#[cynic(graphql_type = "GitObject")]
pub enum TreeObject {
    Tree(Tree),
    #[cynic(fallback)]
    Unknown,
}

impl TreeObject {
    pub fn into_tree_entries(self) -> Option<Vec<TreeEntry>> {
        match self {
            Self::Tree(tree) => Some(tree.entries),
            Self::Unknown => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::github::github_client::{MICROSOFT, WINGET_PKGS};
    use crate::github::graphql::get_directory_content::GetDirectoryContentVariables;
    use crate::github::graphql::get_directory_content_with_text::GetDirectoryContentWithText;
    use cynic::QueryBuilder;
    use indoc::indoc;

    #[test]
    fn get_directory_content_with_text_output() {
        const GET_DIRECTORY_CONTENT_WITH_TEXT_QUERY: &str = indoc! {r#"
            query GetDirectoryContentWithText($owner: String!, $name: String!, $expression: String!) {
              repository(owner: $owner, name: $name) {
                object(expression: $expression) {
                  __typename
                  ... on Tree {
                    entries {
                      name
                      object {
                        __typename
                        ... on Blob {
                          text
                        }
                      }
                    }
                  }
                }
              }
            }
        "#};

        let operation = GetDirectoryContentWithText::build(GetDirectoryContentVariables {
            owner: MICROSOFT,
            name: WINGET_PKGS,
            expression: "",
        });

        assert_eq!(operation.query, GET_DIRECTORY_CONTENT_WITH_TEXT_QUERY);
    }
}
