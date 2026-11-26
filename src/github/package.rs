use std::collections::BTreeSet;

use anstream::println;
use chrono::Local;
use color_eyre::eyre::{Result, bail};
use owo_colors::OwoColorize;
use tokio::try_join;
use winget_types::{PackageIdentifier, PackageVersion};

use crate::{
    commands::utils::environment::CI,
    github::{GitHubError, client::GitHub, graphql::types::PullRequest},
    i18n::fl,
    manifests::Manifests,
    prompts::text::confirm_prompt,
};

pub struct Versioned;

pub struct Unversioned;

pub trait VersionedState<'version> {
    type Version;
}

impl<'version> VersionedState<'version> for Versioned {
    type Version = &'version PackageVersion;
}

impl VersionedState<'_> for Unversioned {
    type Version = ();
}

pub struct Package<'identifier, 'version, V: VersionedState<'version>> {
    identifier: &'identifier PackageIdentifier,
    version: V::Version,
    versions: BTreeSet<PackageVersion>,
    pub manifests: Option<Manifests>,
    existing_pr: Option<PullRequest>,
}

impl<'identifier, 'version, V: VersionedState<'version>> Package<'identifier, 'version, V> {
    /// Returns the package's identifier.
    #[expect(unused)]
    pub const fn identifier(&self) -> &'identifier PackageIdentifier {
        self.identifier
    }
}

impl Package<'_, '_, Versioned> {
    /// Returns the package's versions.
    pub const fn versions(&self) -> &BTreeSet<PackageVersion> {
        &self.versions
    }

    /// Returns the latest version of the package.
    pub fn latest_version(&self) -> &PackageVersion {
        self.versions.last().unwrap_or_else(|| unreachable!())
    }

    pub const fn manifests_mut(&mut self) -> Option<&mut Manifests> {
        self.manifests.as_mut()
    }

    /// Checks for an existing pull request for this package version and returns whether to proceed.
    ///
    /// Interactively, the user is asked whether to proceed. In CI, where prompting isn't possible,
    /// a pull request that was closed without being merged doesn't block a new submission, whereas
    /// an open or merged pull request results in an error so that the caller doesn't mistake an
    /// early exit for a successful submission.
    pub fn prompt_existing_pr(&self) -> Result<bool> {
        let Some(ref pull_request) = self.existing_pr else {
            return Ok(true);
        };

        let created_at = pull_request.created_at.with_timezone(&Local);
        println!(
            "{}",
            fl!(
                "pull-request-exists",
                state = pull_request.state.id(),
                identifier = self.identifier.as_str(),
                version = self.version.to_string(),
                date = created_at.date_naive().to_string(),
                time = created_at.time().to_string()
            )
        );
        println!("{}", pull_request.url.blue());

        if *CI {
            if pull_request.state.is_closed() {
                println!("{}", fl!("pull-request-closed-proceeding"));
                return Ok(true);
            }

            bail!(
                "{}",
                fl!(
                    "error-pull-request-exists",
                    state = pull_request.state.id(),
                    identifier = self.identifier.as_str(),
                    version = self.version.to_string()
                )
            );
        }

        Ok(confirm_prompt(&fl!("prompt-proceed"))?)
    }
}

impl<'identifier> Package<'identifier, '_, Unversioned> {
    pub async fn into_versioned<'version, 'a>(
        self,
        version: &'version PackageVersion,
        github: &'a GitHub,
    ) -> Result<Package<'identifier, 'version, Versioned>, GitHubError> {
        let existing_pr = github
            .get_existing_pull_request(self.identifier, version)
            .await?;

        Ok(Package {
            identifier: self.identifier,
            version,
            manifests: self.manifests,
            versions: self.versions,
            existing_pr,
        })
    }

    /// Returns the latest version of the package, if the package exists.
    pub fn latest_version(&self) -> Option<&PackageVersion> {
        self.versions.last()
    }
}

impl GitHub {
    pub async fn get_versioned_package<'identifier, 'version>(
        &self,
        identifier: &'identifier PackageIdentifier,
        version: &'version PackageVersion,
    ) -> Result<Package<'identifier, 'version, Versioned>, GitHubError> {
        let (versions, existing_pr) = try_join!(
            self.get_versions(identifier),
            self.get_existing_pull_request(identifier, version),
        )?;

        Ok(Package {
            identifier,
            version,
            manifests: if let Some(version) = versions.last() {
                Some(self.get_manifests(identifier, version).await?)
            } else {
                None
            },
            versions,
            existing_pr,
        })
    }

    /// Fetches an unversioned [Package] which may not exist in `winget-pkgs`.
    pub async fn get_package<'identifier>(
        &self,
        identifier: &'identifier PackageIdentifier,
    ) -> Result<Package<'identifier, '_, Unversioned>, GitHubError> {
        let versions = match self.get_versions(identifier).await {
            Ok(versions) => versions,
            Err(GitHubError::PackageNonExistent(_)) => BTreeSet::default(),
            Err(err) => return Err(err),
        };

        Ok(Package {
            identifier,
            version: (),
            manifests: if let Some(version) = versions.last() {
                Some(self.get_manifests(identifier, version).await?)
            } else {
                None
            },
            versions,
            existing_pr: None,
        })
    }
}
