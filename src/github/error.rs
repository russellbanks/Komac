use color_eyre::{Report, eyre};
use cynic::http::CynicReqwestError;
use thiserror::Error;
use winget_types::{ManifestType, PackageIdentifier};

use super::{rest::RestError, upstream, utils::PackagePath};
use crate::i18n::fl;

#[derive(Debug, Error)]
pub enum GitHubError {
    #[error(transparent)]
    GraphQL(eyre::Error),
    #[error(transparent)]
    Rest(#[from] RestError),
    #[error("{}", fl!(
        "error-package-not-found",
        identifier = .0.as_str(),
        upstream = upstream().to_string()
    ))]
    PackageNonExistent(PackageIdentifier),
    #[error("{}", fl!(
        "error-manifest-not-found",
        manifest_type = .r#type.to_string(),
        path = .path.as_str()
    ))]
    ManifestNotFound {
        r#type: ManifestType,
        path: PackagePath,
    },
    #[error("{}", fl!("error-no-valid-files", path = .path.as_str()))]
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
                .fold(err, Report::wrap_err),
        )
    }
}
