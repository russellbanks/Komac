use crate::github::graphql::github_schema::github_schema as schema;

/// <https://docs.github.com/graphql/reference/queries#viewer>
#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Query")]
pub struct GetCurrentUserLogin {
    pub viewer: User,
}

/// <https://docs.github.com/graphql/reference/objects#user>
#[derive(cynic::QueryFragment)]
pub struct User {
    pub login: String,
}

#[cfg(test)]
mod tests {
    use crate::github::graphql::get_current_user_login::GetCurrentUserLogin;
    use cynic::QueryBuilder;
    use indoc::indoc;

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
