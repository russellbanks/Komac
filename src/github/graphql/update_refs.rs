use crate::github::graphql::github_schema::github_schema as schema;
use crate::github::graphql::types::{GitObjectId, GitRefName};

/// <https://docs.github.com/graphql/reference/input-objects#updaterefsinput>
#[derive(cynic::QueryVariables)]
pub struct UpdateRefsVariables<'id> {
    pub ref_updates: Vec<RefUpdate>,
    pub repository_id: &'id cynic::Id,
}

#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Mutation", variables = "UpdateRefsVariables")]
pub struct UpdateRefs {
    #[allow(dead_code)]
    #[arguments(input: { refUpdates: $ref_updates, repositoryId: $repository_id })]
    pub update_refs: Option<UpdateRefsPayload>,
}

#[derive(cynic::QueryFragment)]
pub struct UpdateRefsPayload {
    #[allow(dead_code)]
    pub client_mutation_id: Option<String>,
}

/// <https://docs.github.com/graphql/reference/input-objects#refupdate>
#[derive(cynic::InputObject)]
pub struct RefUpdate {
    pub after_oid: GitObjectId,
    #[cynic(skip_serializing_if = "Option::is_none")]
    pub before_oid: Option<GitObjectId>,
    #[cynic(skip_serializing_if = "Option::is_none")]
    pub force: Option<bool>,
    pub name: GitRefName,
}

#[cfg(test)]
mod tests {
    use crate::github::graphql::update_refs::{UpdateRefs, UpdateRefsVariables};
    use cynic::{Id, MutationBuilder};
    use indoc::indoc;

    #[test]
    fn create_ref_output() {
        const UPDATE_REFS_MUTATION: &str = indoc! {"
            mutation UpdateRefs($refUpdates: [RefUpdate!]!, $repositoryId: ID!) {
              updateRefs(input: {refUpdates: $refUpdates, repositoryId: $repositoryId, }) {
                clientMutationId
              }
            }

        "};

        let id = Id::new("");
        let operation = UpdateRefs::build(UpdateRefsVariables {
            repository_id: &id,
            ref_updates: vec![],
        });

        assert_eq!(operation.query, UPDATE_REFS_MUTATION);
    }
}
