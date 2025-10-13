use bon::Builder;
use const_format::str_repeat;
use ref_update_builder::SetAfterOid;

use super::{
    github_schema as schema,
    types::{GitObjectId, GitRefName},
};

/// <https://docs.github.com/graphql/reference/input-objects#updaterefsinput>
#[derive(cynic::QueryVariables)]
pub struct UpdateRefsInput<'id> {
    /// A unique identifier for the client performing the mutation.
    #[cynic(skip_serializing_if = "Option::is_none")]
    client_mutation_id: Option<String>,

    /// A list of ref updates.
    ref_updates: Vec<RefUpdate>,

    /// The Node ID of the repository.
    repository_id: &'id cynic::Id,
}

impl<'id> UpdateRefsInput<'id> {
    #[inline]
    pub const fn new(ref_updates: Vec<RefUpdate>, repository_id: &'id cynic::Id) -> Self {
        Self {
            client_mutation_id: None,
            ref_updates,
            repository_id,
        }
    }
}

/// Creates, updates and/or deletes multiple refs in a repository.
///
/// This mutation takes a list of [`RefUpdates`] and performs these updates on the repository. All
/// updates are performed atomically, meaning that if one of them is rejected, no other ref will be
/// modified.
///
/// `RefUpdate.beforeOid` specifies that the given reference needs to point to the given value
/// before performing any updates. A value of `0000000000000000000000000000000000000000` can be used
/// to verify that the references should not exist.
///
/// `RefUpdate.afterOid` specifies the value that the given reference will point to after
/// performing all updates. A value of `0000000000000000000000000000000000000000` can be used to
/// delete a reference.
///
/// If `RefUpdate.force` is set to `true`, a non-fast-forward updates for the given reference will
/// be allowed.
///
/// See <https://docs.github.com/en/graphql/reference/mutations#updaterefs>.
///
/// [`RefUpdates`]: RefUpdate
#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Mutation", variables = "UpdateRefsInput")]
pub struct UpdateRefs {
    #[expect(dead_code)]
    #[arguments(input: { clientMutationId: $client_mutation_id, refUpdates: $ref_updates, repositoryId: $repository_id })]
    pub update_refs: Option<UpdateRefsPayload>,
}

#[derive(cynic::QueryFragment)]
pub struct UpdateRefsPayload {
    /// A unique identifier for the client performing the mutation.
    #[expect(dead_code)]
    pub client_mutation_id: Option<String>,
}

/// <https://docs.github.com/graphql/reference/input-objects#refupdate>
#[derive(Builder, cynic::InputObject)]
pub struct RefUpdate {
    /// The value this ref should be updated to.
    #[builder(into)]
    after_oid: GitObjectId,

    /// The value this ref needs to point to before the update.
    #[cynic(skip_serializing_if = "Option::is_none")]
    before_oid: Option<GitObjectId>,

    /// Force a non fast-forward update.
    #[cynic(skip_serializing_if = "Option::is_none")]
    force: Option<bool>,

    /// The fully qualified name of the ref to be updated.
    ///
    /// For example, `refs/heads/branch-name`.
    #[builder(into)]
    name: GitRefName,
}

impl RefUpdate {
    pub fn delete() -> RefUpdateBuilder<SetAfterOid> {
        const DELETE_ID: &str = str_repeat!("0", 40);

        Self::builder().after_oid(DELETE_ID)
    }

    pub fn delete_branch<T: Into<String>>(branch_name: T) -> Self {
        Self::delete()
            .name(GitRefName::new_branch(branch_name))
            .build()
    }

    pub fn delete_branches<I, T>(branches: I) -> Vec<Self>
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        branches.into_iter().map(Self::delete_branch).collect()
    }
}

#[cfg(test)]
mod tests {
    use cynic::{Id, MutationBuilder};
    use indoc::indoc;

    use super::{UpdateRefs, UpdateRefsInput};

    #[test]
    fn create_ref_output() {
        const UPDATE_REFS_MUTATION: &str = indoc! {"
            mutation UpdateRefs($clientMutationId: String, $refUpdates: [RefUpdate!]!, $repositoryId: ID!) {
              updateRefs(input: {clientMutationId: $clientMutationId, refUpdates: $refUpdates, repositoryId: $repositoryId}) {
                clientMutationId
              }
            }
        "};

        let id = Id::new("");
        let operation = UpdateRefs::build(UpdateRefsInput {
            client_mutation_id: None,
            repository_id: &id,
            ref_updates: vec![],
        });

        assert_eq!(operation.query, UPDATE_REFS_MUTATION);
    }
}
