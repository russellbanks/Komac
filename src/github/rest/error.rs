use color_eyre::eyre::eyre;
use serde::Deserialize;

use super::super::GitHubError;

#[derive(Debug, Deserialize)]
pub struct RestError {
    pub status: Option<String>,
    pub message: String,
}

impl From<RestError> for GitHubError {
    fn from(value: RestError) -> Self {
        let RestError { status, message } = value;
        Self::Rest(match status {
            Some(status) => eyre!("Status {status}: {message}"),
            None => eyre!("{message}"),
        })
    }
}
