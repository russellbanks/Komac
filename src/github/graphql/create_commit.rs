use crate::github::graphql::github_schema::github_schema as schema;
use crate::github::graphql::types::{Base64String, GitObjectId};
use derive_new::new;
use std::borrow::Cow;
use url::Url;

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
#[derive(cynic::InputObject)]
pub struct CreateCommitOnBranchInput<'a> {
    pub branch: CommittableBranch<'a>,
    pub expected_head_oid: GitObjectId,
    #[cynic(skip_serializing_if = "Option::is_none")]
    pub file_changes: Option<FileChanges<'a>>,
    pub message: CommitMessage<'a>,
}

/// <https://docs.github.com/graphql/reference/input-objects#filechanges>
#[derive(cynic::InputObject)]
pub struct FileChanges<'a> {
    #[cynic(skip_serializing_if = "Option::is_none")]
    pub additions: Option<Vec<FileAddition<'a>>>,
    #[cynic(skip_serializing_if = "Option::is_none")]
    pub deletions: Option<Vec<FileDeletion<'a>>>,
}

/// <https://docs.github.com/graphql/reference/input-objects#filedeletion>
#[derive(cynic::InputObject, new)]
pub struct FileDeletion<'path> {
    #[new(into)]
    pub path: Cow<'path, str>,
}

/// <https://docs.github.com/graphql/reference/input-objects#fileaddition>
#[derive(cynic::InputObject, new)]
pub struct FileAddition<'path> {
    pub contents: Base64String,
    #[new(into)]
    pub path: Cow<'path, str>,
}

/// <https://docs.github.com/graphql/reference/input-objects#committablebranch>
#[derive(cynic::InputObject)]
pub struct CommittableBranch<'a> {
    pub id: &'a cynic::Id,
}

/// <https://docs.github.com/graphql/reference/input-objects#commitmessage>
#[derive(cynic::InputObject)]
pub struct CommitMessage<'a> {
    #[cynic(skip_serializing_if = "Option::is_none")]
    pub body: Option<&'a str>,
    pub headline: &'a str,
}

#[cfg(test)]
mod tests {
    use crate::github::graphql::create_commit::{
        CommitMessage, CommittableBranch, CreateCommit, CreateCommitOnBranchInput,
        CreateCommitVariables,
    };
    use crate::github::graphql::types::GitObjectId;
    use cynic::{Id, MutationBuilder};
    use indoc::indoc;

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
            input: CreateCommitOnBranchInput {
                branch: CommittableBranch { id: &id },
                expected_head_oid: GitObjectId::new(""),
                file_changes: None,
                message: CommitMessage {
                    body: None,
                    headline: "",
                },
            },
        });

        assert_eq!(operation.query, CREATE_COMMIT_MUTATION);
    }
}
