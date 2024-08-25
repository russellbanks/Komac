use crate::github::graphql::github_schema::github_schema as schema;
use crate::github::graphql::types::GitObjectId;

/// <https://docs.github.com/graphql/reference/input-objects#createrefinput>
#[derive(cynic::QueryVariables)]
pub struct CreateRefVariables<'a> {
    pub name: &'a str,
    pub oid: GitObjectId,
    pub repository_id: &'a cynic::Id,
}

#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Mutation", variables = "CreateRefVariables")]
pub struct CreateRef {
    #[arguments(input: { name: $name, oid: $oid, repositoryId: $repository_id })]
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

#[cfg(test)]
mod tests {
    use crate::github::graphql::create_ref::{CreateRef, CreateRefVariables};
    use crate::github::graphql::types::GitObjectId;
    use cynic::{Id, MutationBuilder};
    use indoc::indoc;

    #[test]
    fn create_ref_output() {
        const CREATE_REF_MUTATION: &str = indoc! {"
            mutation CreateRef($name: String!, $oid: GitObjectID!, $repositoryId: ID!) {
              createRef(input: {name: $name, oid: $oid, repositoryId: $repositoryId, }) {
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
        let operation = CreateRef::build(CreateRefVariables {
            name: "",
            oid: GitObjectId::new(""),
            repository_id: &id,
        });

        assert_eq!(operation.query, CREATE_REF_MUTATION);
    }
}
