use std::env;

use color_eyre::eyre::eyre;
use cynic::{GraphQlResponse, QueryBuilder, http::ReqwestExt};

use super::{
    super::{GitHubError, client::GitHub},
    GRAPHQL_URL, github_schema as schema,
};

/// <https://docs.github.com/graphql/reference/queries#viewer>
#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Query")]
pub struct GetCurrentUserLogin {
    /// The currently authenticated user.
    viewer: User,
}

impl GetCurrentUserLogin {
    /// Returns the username used to login.
    #[inline]
    pub fn username(self) -> String {
        self.viewer.login
    }
}

/// A user is an individual's account on GitHub that owns repositories and can make new content.
///
/// See <https://docs.github.com/graphql/reference/objects#user>.
#[derive(cynic::QueryFragment)]
pub struct User {
    /// The username used to login.
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

            let Some(data) = data else {
                return Err(GitHubError::graphql_errors(
                    eyre!("failed to get username of current user"),
                    errors,
                ));
            };

            Ok(data.username())
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
