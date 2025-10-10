use color_eyre::eyre::eyre;
use cynic::{GraphQlResponse, Id, MutationBuilder, http::ReqwestExt};

use super::{GRAPHQL_URL, github_schema as schema, types::GitObjectId};
use crate::github::{GitHubError, client::GitHub};

#[derive(cynic::QueryVariables)]
pub struct MergeUpstreamVariables<'id> {
    branch_ref_id: &'id Id,
    upstream_target_oid: GitObjectId,
    force: bool,
}

impl<'id> MergeUpstreamVariables<'id> {
    #[inline]
    pub const fn new(
        branch_ref_id: &'id Id,
        upstream_target_oid: GitObjectId,
        force: bool,
    ) -> Self {
        Self {
            branch_ref_id,
            upstream_target_oid,
            force,
        }
    }
}

#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Mutation", variables = "MergeUpstreamVariables")]
pub struct UpdateRef {
    #[expect(dead_code)]
    #[arguments(input: { oid: $upstream_target_oid, refId: $branch_ref_id, force: $force })]
    pub update_ref: Option<UpdateRefPayload>,
}

#[derive(cynic::QueryFragment)]
pub struct UpdateRefPayload {
    #[expect(dead_code)]
    pub client_mutation_id: Option<String>,
}

impl GitHub {
    pub async fn merge_upstream(
        &self,
        branch_ref_id: &Id,
        upstream_target_oid: GitObjectId,
        force: bool,
    ) -> Result<(), GitHubError> {
        let GraphQlResponse { data, errors } = self
            .0
            .post(GRAPHQL_URL)
            .run_graphql(UpdateRef::build(MergeUpstreamVariables::new(
                branch_ref_id,
                upstream_target_oid,
                force,
            )))
            .await?;

        if data.is_some() {
            Ok(())
        } else {
            Err(GitHubError::graphql_errors(
                eyre!("failed to sync upstream changes"),
                errors,
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use cynic::{Id, MutationBuilder};
    use indoc::indoc;

    use super::{super::types::GitObjectId, MergeUpstreamVariables, UpdateRef};

    #[test]
    fn update_ref_output() {
        const UPDATE_REF_MUTATION: &str = indoc! {"
            mutation UpdateRef($branchRefId: ID!, $upstreamTargetOid: GitObjectID!, $force: Boolean!) {
              updateRef(input: {oid: $upstreamTargetOid, refId: $branchRefId, force: $force}) {
                clientMutationId
              }
            }
        "};

        let id = Id::new("");
        let operation = UpdateRef::build(MergeUpstreamVariables {
            branch_ref_id: &id,
            upstream_target_oid: GitObjectId::new(""),
            force: false,
        });

        assert_eq!(operation.query, UPDATE_REF_MUTATION);
    }
}
