use bon::Builder;

use super::{schema::github_schema as schema, types::GitObjectId};

/// See <https://docs.github.com/graphql/reference/input-objects#createrefinput>.
#[derive(Builder, cynic::QueryVariables)]
pub struct CreateRefVariables<'a> {
    /// A unique identifier for the client performing the mutation.
    client_mutation_id: Option<String>,

    /// The fully qualified name of the new Ref (ie: `refs/heads/my_new_branch`).
    name: &'a str,

    /// The `GitObjectID` that the new Ref shall target. Must point to a commit.
    #[builder(into)]
    oid: GitObjectId,

    /// The Node ID of the Repository to create the Ref in.
    repository_id: &'a cynic::Id,
}

#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Mutation", variables = "CreateRefVariables")]
pub struct CreateRef {
    #[arguments(input: { clientMutationId: $client_mutation_id, name: $name, oid: $oid, repositoryId: $repository_id })]
    pub create_ref: Option<CreateRefPayload>,
}

/// <https://docs.github.com/graphql/reference/mutations#createref>
#[derive(cynic::QueryFragment)]
pub struct CreateRefPayload {
    #[cynic(rename = "ref")]
    pub ref_: Option<Ref>,
}

/// <https://docs.github.com/graphql/reference/objects#ref>
#[derive(cynic::QueryFragment)]
pub struct Ref {
    pub id: cynic::Id,
    pub name: String,
    pub target: Option<GitObject>,
}

/// <https://docs.github.com/graphql/reference/interfaces#gitobject>
#[derive(cynic::QueryFragment)]
pub struct GitObject {
    pub oid: GitObjectId,
}

impl From<GitObject> for GitObjectId {
    #[inline]
    fn from(object: GitObject) -> Self {
        object.oid
    }
}

#[cfg(test)]
mod tests {
    use cynic::{Id, MutationBuilder};
    use indoc::indoc;

    use super::{CreateRef, CreateRefVariables};

    #[test]
    fn create_ref_output() {
        const CREATE_REF_MUTATION: &str = indoc! {"
            mutation CreateRef($clientMutationId: String, $name: String!, $oid: GitObjectID!, $repositoryId: ID!) {
              createRef(input: {clientMutationId: $clientMutationId, name: $name, oid: $oid, repositoryId: $repositoryId}) {
                ref {
                  id
                  name
                  target {
                    oid
                  }
                }
              }
            }
        "};

        let id = Id::new("");
        let operation = CreateRef::build(
            CreateRefVariables::builder()
                .name("")
                .oid("")
                .repository_id(&id)
                .build(),
        );

        assert_eq!(operation.query, CREATE_REF_MUTATION);
    }
}
