use std::borrow::Cow;

use bon::{Builder, bon};
use color_eyre::eyre::eyre;
use cynic::{GraphQlResponse, Id, MutationBuilder, http::ReqwestExt};
use url::Url;

use super::{
    super::{GitHubError, client::GitHub},
    GRAPHQL_URL, github_schema as schema,
    types::{Base64String, GitObjectId},
};

#[derive(cynic::QueryVariables)]
pub struct CreateCommitVariables<'a> {
    pub input: CreateCommitOnBranchInput<'a>,
}

#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Mutation", variables = "CreateCommitVariables")]
pub struct CreateCommit {
    #[arguments(input: $input)]
    pub create_commit_on_branch: Option<CreateCommitOnBranchPayload>,
}

/// <https://docs.github.com/graphql/reference/mutations#createcommitonbranch>
#[derive(cynic::QueryFragment)]
pub struct CreateCommitOnBranchPayload {
    pub commit: Option<Commit>,
}

/// <https://docs.github.com/graphql/reference/objects#commit>
#[derive(cynic::QueryFragment)]
pub struct Commit {
    pub url: Url,
}

/// <https://docs.github.com/graphql/reference/input-objects#createcommitonbranchinput>
#[derive(Builder, cynic::InputObject)]
pub struct CreateCommitOnBranchInput<'a> {
    branch: CommittableBranch<'a>,
    client_mutation_id: Option<&'a str>,
    expected_head_oid: GitObjectId,
    #[cynic(skip_serializing_if = "Option::is_none")]
    file_changes: Option<FileChanges<'a>>,
    message: CommitMessage<'a>,
}

/// <https://docs.github.com/graphql/reference/input-objects#filechanges>
#[derive(cynic::InputObject)]
pub struct FileChanges<'a> {
    #[cynic(skip_serializing_if = "Vec::is_empty")]
    pub additions: Vec<FileAddition<'a>>,

    #[cynic(skip_serializing_if = "Vec::is_empty")]
    pub deletions: Vec<FileDeletion<'a>>,
}

impl<'a> FileChanges<'a> {
    #[inline]
    pub const fn new(additions: Vec<FileAddition<'a>>, deletions: Vec<FileDeletion<'a>>) -> Self {
        Self {
            additions,
            deletions,
        }
    }
}

/// <https://docs.github.com/graphql/reference/input-objects#filedeletion>
#[derive(cynic::InputObject)]
pub struct FileDeletion<'path> {
    pub path: Cow<'path, str>,
}

impl<'path> FileDeletion<'path> {
    pub fn new<P: Into<Cow<'path, str>>>(path: P) -> Self {
        Self { path: path.into() }
    }
}

/// <https://docs.github.com/graphql/reference/input-objects#fileaddition>
#[derive(cynic::InputObject)]
pub struct FileAddition<'path> {
    pub path: Cow<'path, str>,
    pub contents: Base64String,
}

impl<'path> FileAddition<'path> {
    pub fn new<P, T>(path: P, contents: T) -> Self
    where
        P: Into<Cow<'path, str>>,
        T: AsRef<[u8]>,
    {
        Self {
            path: path.into(),
            contents: Base64String::encode_string(contents.as_ref()),
        }
    }
}

/// <https://docs.github.com/graphql/reference/input-objects#committablebranch>
#[derive(cynic::InputObject)]
pub struct CommittableBranch<'a> {
    pub id: &'a Id,
}

impl<'a> CommittableBranch<'a> {
    pub fn new<T: Into<&'a Id>>(id: T) -> Self {
        Self { id: id.into() }
    }
}

/// <https://docs.github.com/graphql/reference/input-objects#commitmessage>
#[derive(cynic::InputObject)]
pub struct CommitMessage<'a> {
    #[cynic(skip_serializing_if = "Option::is_none")]
    pub body: Option<&'a str>,
    pub headline: &'a str,
}

impl<'a> CommitMessage<'a> {
    #[inline]
    pub const fn new_headline(headline: &'a str) -> Self {
        Self {
            body: None,
            headline,
        }
    }
}

#[bon]
impl GitHub {
    #[builder(finish_fn = create)]
    pub async fn commit(
        &self,
        branch_id: &Id,
        #[builder(into)] head_sha: GitObjectId,
        message: &str,
        #[builder(default)] additions: Vec<FileAddition<'_>>,
        #[builder(default)] deletions: Vec<FileDeletion<'_>>,
    ) -> Result<Url, GitHubError> {
        let GraphQlResponse { data, errors } = self
            .0
            .post(GRAPHQL_URL)
            .run_graphql(CreateCommit::build(CreateCommitVariables {
                input: CreateCommitOnBranchInput::builder()
                    .branch(CommittableBranch::new(branch_id))
                    .expected_head_oid(head_sha)
                    .file_changes(FileChanges::new(additions, deletions))
                    .message(CommitMessage::new_headline(message))
                    .build(),
            }))
            .await?;

        data.and_then(|data| data.create_commit_on_branch?.commit)
            .map(|commit| commit.url)
            .ok_or_else(|| GitHubError::graphql_errors(eyre!("failed to create commit"), errors))
    }
}

#[cfg(test)]
mod tests {
    use cynic::{Id, MutationBuilder};
    use indoc::indoc;

    use crate::github::graphql::{
        create_commit::{
            CommitMessage, CommittableBranch, CreateCommit, CreateCommitOnBranchInput,
            CreateCommitVariables,
        },
        types::GitObjectId,
    };

    #[test]
    fn create_commit_output() {
        const CREATE_COMMIT_MUTATION: &str = indoc! {"
            mutation CreateCommit($input: CreateCommitOnBranchInput!) {
              createCommitOnBranch(input: $input) {
                commit {
                  url
                }
              }
            }
        "};

        let id = Id::new("");
        let operation = CreateCommit::build(CreateCommitVariables {
            input: CreateCommitOnBranchInput::builder()
                .branch(CommittableBranch::new(&id))
                .expected_head_oid(GitObjectId::new(""))
                .message(CommitMessage::new_headline(""))
                .build(),
        });

        assert_eq!(operation.query, CREATE_COMMIT_MUTATION);
    }
}
