use color_eyre::{Report, Section, eyre};
use cynic::http::CynicReqwestError;
use thiserror::Error;
use winget_types::{ManifestType, PackageIdentifier};

use super::utils::PackagePath;

#[derive(Debug, Error)]
pub enum GitHubError {
    #[error(transparent)]
    GraphQL(#[from] eyre::Error),
    #[error("{0} does not exist")]
    PackageNonExistent(PackageIdentifier),
    #[error("No {type} manifest was found in {path}")]
    ManifestNotFound {
        r#type: ManifestType,
        path: PackagePath,
    },
    #[error("No valid files were found for {path}")]
    NoValidFiles { path: PackagePath },
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    CynicRequest(#[from] CynicReqwestError),
    #[error(transparent)]
    YamlError(#[from] serde_yaml::Error),
    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),
}

impl GitHubError {
    pub fn graphql_errors<T, E>(err: eyre::Error, errors: T) -> Self
    where
        T: Into<Option<Vec<E>>>,
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::GraphQL(
            errors
                .into()
                .unwrap_or_default()
                .into_iter()
                .fold(err, Report::error),
        )
    }
}
