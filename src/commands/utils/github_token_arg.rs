use std::borrow::Cow;

use clap::Args;
use secrecy::SecretString;

use crate::token::{TokenError, TokenManager};

#[derive(Args)]
pub struct GitHubTokenArg {
    /// GitHub personal access token with the `public_repo` scope
    #[arg(short, long, env = "GITHUB_TOKEN", hide_env_values = true)]
    token: Option<SecretString>,
}

impl GitHubTokenArg {
    pub async fn resolve(&self) -> Result<Cow<'_, SecretString>, TokenError> {
        TokenManager::handle(&self.token).await
    }
}
