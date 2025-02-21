use crate::github::graphql::{github_schema::github_schema as schema, types::GitObjectId};

#[derive(cynic::QueryVariables)]
pub struct MergeUpstreamVariables<'id> {
    pub branch_ref_id: &'id cynic::Id,
    pub upstream_target_oid: GitObjectId,
    pub force: bool,
}

#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Mutation", variables = "MergeUpstreamVariables")]
pub struct MergeUpstream {
    #[expect(dead_code)]
    #[arguments(input: { oid: $upstream_target_oid, refId: $branch_ref_id, force: $force })]
    pub update_ref: Option<UpdateRefPayload>,
}

#[derive(cynic::QueryFragment)]
pub struct UpdateRefPayload {
    #[expect(dead_code)]
    pub client_mutation_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use cynic::{Id, MutationBuilder};
    use indoc::indoc;

    use crate::github::graphql::{
        merge_upstream::{MergeUpstream, MergeUpstreamVariables},
        types::GitObjectId,
    };

    #[test]
    fn merge_upstream_output() {
        const MERGE_UPSTREAM_MUTATION: &str = indoc! {"
            mutation MergeUpstream($branchRefId: ID!, $upstreamTargetOid: GitObjectID!, $force: Boolean!) {
              updateRef(input: {oid: $upstreamTargetOid, refId: $branchRefId, force: $force}) {
                clientMutationId
              }
            }
        "};

        let id = Id::new("");
        let operation = MergeUpstream::build(MergeUpstreamVariables {
            branch_ref_id: &id,
            upstream_target_oid: GitObjectId::new(""),
            force: false,
        });

        assert_eq!(operation.query, MERGE_UPSTREAM_MUTATION);
    }
}
