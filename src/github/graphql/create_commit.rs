use crate::github::graphql::get_repository_info::GitObjectId;
use crate::github::graphql::github_schema::github_schema as schema;
use url::Url;

/*
mutation CreateCommit($input: CreateCommitOnBranchInput!) {
  createCommitOnBranch(input: $input) {
    commit {
      url
    }
  }
}
*/

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

#[derive(cynic::QueryFragment)]
pub struct CreateCommitOnBranchPayload {
    pub commit: Option<Commit>,
}

#[derive(cynic::QueryFragment)]
pub struct Commit {
    pub url: Url,
}

#[derive(cynic::InputObject)]
pub struct CreateCommitOnBranchInput<'a> {
    pub branch: CommittableBranch<'a>,
    #[cynic(skip_serializing_if = "Option::is_none")]
    pub client_mutation_id: Option<&'a str>,
    pub expected_head_oid: GitObjectId,
    #[cynic(skip_serializing_if = "Option::is_none")]
    pub file_changes: Option<FileChanges<'a>>,
    pub message: CommitMessage<'a>,
}

#[derive(cynic::InputObject)]
pub struct FileChanges<'a> {
    #[cynic(skip_serializing_if = "Option::is_none")]
    pub additions: Option<Vec<FileAddition<'a>>>,
    #[cynic(skip_serializing_if = "Option::is_none")]
    pub deletions: Option<Vec<FileDeletion<'a>>>,
}

#[derive(cynic::InputObject)]
pub struct FileDeletion<'a> {
    pub path: &'a str,
}

#[derive(cynic::InputObject)]
pub struct FileAddition<'a> {
    pub contents: Base64String,
    pub path: &'a str,
}

#[derive(cynic::InputObject)]
pub struct CommittableBranch<'a> {
    #[cynic(skip_serializing_if = "Option::is_none")]
    pub branch_name: Option<&'a str>,
    #[cynic(skip_serializing_if = "Option::is_none")]
    pub id: Option<&'a cynic::Id>,
    #[cynic(skip_serializing_if = "Option::is_none")]
    pub repository_name_with_owner: Option<&'a str>,
}

#[derive(cynic::InputObject)]
pub struct CommitMessage<'a> {
    #[cynic(skip_serializing_if = "Option::is_none")]
    pub body: Option<&'a str>,
    pub headline: &'a str,
}

#[derive(cynic::Scalar)]
pub struct Base64String(pub String);
