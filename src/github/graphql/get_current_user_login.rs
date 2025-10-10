use std::env;

use cynic::{GraphQlResponse, QueryBuilder, http::ReqwestExt};

use super::{
    super::github_client::{GitHub, GitHubError},
    GRAPHQL_URL, github_schema as schema,
};

/// <https://docs.github.com/graphql/reference/queries#viewer>
#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Query")]
pub struct GetCurrentUserLogin {
    viewer: User,
}

impl GetCurrentUserLogin {
    #[inline]
    pub fn username(self) -> String {
        self.viewer.login
    }
}

/// <https://docs.github.com/graphql/reference/objects#user>
#[derive(cynic::QueryFragment)]
pub struct User {
    login: String,
}

impl GitHub {
    pub async fn get_username(&self) -> Result<String, GitHubError> {
        const KOMAC_FORK_OWNER: &str = "KOMAC_FORK_OWNER";

        if let Ok(login) = env::var(KOMAC_FORK_OWNER) {
            Ok(login)
        } else {
            let GraphQlResponse { data, errors } = self
                .0
                .post(GRAPHQL_URL)
                .run_graphql(GetCurrentUserLogin::build(()))
                .await?;

            data.map(GetCurrentUserLogin::username)
                .ok_or_else(|| GitHubError::GraphQL(errors.unwrap_or_default()))
        }
    }
}

#[cfg(test)]
mod tests {
    use cynic::QueryBuilder;
    use indoc::indoc;

    use super::GetCurrentUserLogin;

    #[test]
    fn get_current_user_login_output() {
        const GET_CURRENT_USER_LOGIN_QUERY: &str = indoc! {r#"
            query GetCurrentUserLogin {
              viewer {
                login
              }
            }
        "#};

        let operation = GetCurrentUserLogin::build(());

        assert_eq!(operation.query, GET_CURRENT_USER_LOGIN_QUERY);
    }
}
