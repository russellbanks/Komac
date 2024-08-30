use crate::github::graphql::get_directory_content::GetDirectoryContentVariablesFields;
use crate::github::graphql::get_directory_content_with_text::BlobObject;
use crate::github::graphql::github_schema::github_schema as schema;

#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Query", variables = "GetDirectoryContentVariables")]
pub struct GetFileContent {
    #[arguments(owner: $owner, name: $name)]
    pub repository: Option<Repository>,
}

#[derive(cynic::QueryFragment)]
#[cynic(variables = "GetDirectoryContentVariables")]
pub struct Repository {
    #[arguments(expression: $expression)]
    pub object: Option<BlobObject>,
}

#[cfg(test)]
mod tests {
    use crate::github::github_client::{MICROSOFT, WINGET_PKGS};
    use crate::github::graphql::get_directory_content::GetDirectoryContentVariables;
    use crate::github::graphql::get_file_content::GetFileContent;
    use cynic::QueryBuilder;
    use indoc::indoc;

    #[test]
    fn get_file_content_output() {
        const GET_FILE_CONTENT_QUERY: &str = indoc! {r#"
            query GetFileContent($owner: String!, $name: String!, $expression: String!) {
              repository(owner: $owner, name: $name) {
                object(expression: $expression) {
                  __typename
                  ... on Blob {
                    text
                  }
                }
              }
            }
        "#};

        let operation = GetFileContent::build(GetDirectoryContentVariables {
            owner: MICROSOFT,
            name: WINGET_PKGS,
            expression: "",
        });

        assert_eq!(operation.query, GET_FILE_CONTENT_QUERY);
    }
}
