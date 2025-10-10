use std::fmt;

use bon::bon;
use color_eyre::eyre::eyre;
use cynic::{GraphQlResponse, QueryBuilder, http::ReqwestExt};

use super::{
    super::{GitHubError, MICROSOFT, WINGET_PKGS, client::GitHub, utils::PackagePath},
    GRAPHQL_URL, GetFileContent, github_schema as schema,
};

#[derive(cynic::QueryVariables)]
pub struct GetDirectoryContentVariables<'a> {
    owner: &'a str,
    name: &'a str,
    expression: &'a str,
}

impl<'a> GetDirectoryContentVariables<'a> {
    pub fn new<O, N, E>(owner: &'a O, name: &'a N, expression: &'a E) -> Self
    where
        O: AsRef<str>,
        N: AsRef<str>,
        E: AsRef<str>,
    {
        Self {
            owner: owner.as_ref(),
            name: name.as_ref(),
            expression: expression.as_ref(),
        }
    }
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

#[bon]
impl GitHub {
    pub async fn get_file_content<O, R, P>(
        &self,
        owner: O,
        repo: R,
        path: P,
    ) -> Result<String, GitHubError>
    where
        O: AsRef<str>,
        R: AsRef<str>,
        P: fmt::Display,
    {
        let GraphQlResponse { data, errors } = self
            .0
            .post(GRAPHQL_URL)
            .run_graphql(GetFileContent::build(GetDirectoryContentVariables::new(
                &owner,
                &repo,
                &format!("HEAD:{path}"),
            )))
            .await?;

        data.and_then(|data| data.repository?.object?.into_blob_text())
            .ok_or_else(|| GitHubError::graphql_errors(eyre!("failed to get {path}"), errors))
    }

    #[builder]
    pub async fn get_directory_content(
        &self,
        #[builder(default = MICROSOFT)] owner: &str,
        #[builder(default = WINGET_PKGS)] repo: &str,
        #[builder(default = "HEAD")] branch_name: &str,
        path: &PackagePath,
    ) -> Result<impl Iterator<Item = String>, GitHubError> {
        let GraphQlResponse { data, errors } = self
            .0
            .post(GRAPHQL_URL)
            .run_graphql(GetDirectoryContent::build(
                GetDirectoryContentVariables::new(&owner, &repo, &format!("{branch_name}:{path}")),
            ))
            .await?;
        let entries = data
            .and_then(|data| data.repository?.object?.into_entries())
            .ok_or_else(|| {
                GitHubError::graphql_errors(
                    eyre!("failed to get {path} in {branch_name} from {owner}/{repo}"),
                    errors,
                )
            })?;

        Ok(entries.into_iter().filter_map(|entry| entry.path))
    }
}

#[cfg(test)]
mod tests {
    use cynic::QueryBuilder;
    use indoc::indoc;

    use super::{
        super::super::{MICROSOFT, WINGET_PKGS},
        GetDirectoryContent, GetDirectoryContentVariables,
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

        let operation = GetDirectoryContent::build(GetDirectoryContentVariables::new(
            &MICROSOFT,
            &WINGET_PKGS,
            &"",
        ));

        assert_eq!(operation.query, GET_DIRECTORY_CONTENT_QUERY);
    }
}
