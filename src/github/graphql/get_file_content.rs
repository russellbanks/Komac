use crate::github::graphql::{
    get_directory_content::GetDirectoryContentVariablesFields,
    get_directory_content_with_text::BlobObject, github_schema as schema,
};

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
    use cynic::QueryBuilder;
    use indoc::indoc;

    use super::{super::get_directory_content::GetDirectoryContentVariables, GetFileContent};
    use crate::github::{MICROSOFT, WINGET_PKGS};

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

        let operation = GetFileContent::build(GetDirectoryContentVariables::new(
            &MICROSOFT,
            &WINGET_PKGS,
            &"",
        ));

        assert_eq!(operation.query, GET_FILE_CONTENT_QUERY);
    }
}
