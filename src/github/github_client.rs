use crate::credential::get_default_headers;
use crate::github::github_utils::get_full_package_path;
use crate::graphql::{
    create_branch, create_commit, create_pull_request, delete_ref, get_all_values,
    get_all_versions, get_branches, get_current_user_login, get_directory_content,
    get_directory_content_with_text, get_pull_request_from_branch, get_repository_id,
    get_repository_info, Branch, GitHubValues, PullRequest, Ref, RepositoryData,
};
use crate::manifests::default_locale_manifest::DefaultLocaleManifest;
use crate::manifests::installer_manifest::InstallerManifest;
use crate::manifests::locale_manifest::LocaleManifest;
use crate::manifests::version_manifest::VersionManifest;
use crate::types::package_identifier::PackageIdentifier;
use crate::types::package_version::PackageVersion;
use color_eyre::eyre::{eyre, Result};
use const_format::formatcp;
use reqwest::Client;
use url::Url;

pub const MICROSOFT: &str = "Microsoft";
pub const WINGET_PKGS: &str = "winget-pkgs";
pub const WINGET_PKGS_FULL_NAME: &str = formatcp!("{MICROSOFT}/{WINGET_PKGS}");

pub struct GitHub(Client);

impl GitHub {
    pub fn new(token: String) -> Result<GitHub> {
        Ok(GitHub(
            Client::builder()
                .default_headers(get_default_headers(Some(&token)))
                .build()?,
        ))
    }

    pub async fn get_versions(&self, path: &str) -> Result<Vec<PackageVersion>> {
        get_all_versions(&self.0, MICROSOFT, WINGET_PKGS, path).await
    }

    pub async fn get_manifests(
        &self,
        identifier: &PackageIdentifier,
        latest_version: &PackageVersion,
    ) -> Result<Manifests> {
        let full_package_path = get_full_package_path(identifier, latest_version);
        let content =
            get_directory_content_with_text(&self.0, MICROSOFT, WINGET_PKGS, &full_package_path)
                .await?;

        let version_manifest = content
            .iter()
            .find(|file| file.name == format!("{identifier}.yaml"))
            .map(|file| serde_yaml::from_str::<VersionManifest>(&file.text))
            .transpose()?
            .ok_or_else(|| eyre!("No version manifest was found in {full_package_path}"))?;

        let locale_manifests = content
            .iter()
            .filter(|file| {
                file.name.starts_with(&format!("{identifier}.locale."))
                    && file.name.ends_with(".yaml")
                    && !file.name.contains(version_manifest.default_locale.as_str())
            })
            .map(|file| serde_yaml::from_str::<LocaleManifest>(&file.text).unwrap())
            .collect::<Vec<_>>();

        let default_locale_manifest = content
            .iter()
            .find(|file| {
                file.name
                    == format!(
                        "{identifier}.locale.{}.yaml",
                        version_manifest.default_locale
                    )
            })
            .map(|file| serde_yaml::from_str::<DefaultLocaleManifest>(&file.text))
            .transpose()?
            .ok_or_else(|| eyre!("No default locale manifest was found in {full_package_path}"))?;

        let installer_manifest = content
            .into_iter()
            .find(|file| file.name == format!("{identifier}.installer.yaml"))
            .map(|file| serde_yaml::from_str::<InstallerManifest>(&file.text))
            .transpose()?
            .ok_or_else(|| eyre!("No installer manifest was found in {full_package_path}"))?;

        Ok(Manifests {
            installer_manifest,
            default_locale_manifest,
            version_manifest,
            locale_manifests,
        })
    }

    pub async fn get_username(&self) -> Result<String> {
        get_current_user_login(&self.0).await
    }

    pub async fn get_winget_pkgs(&self) -> Result<RepositoryData> {
        get_repository_info(&self.0, MICROSOFT, WINGET_PKGS).await
    }

    pub async fn get_winget_pkgs_fork_id(&self, username: &str) -> Result<String> {
        get_repository_id(&self.0, username, WINGET_PKGS).await
    }

    pub async fn create_branch(&self, fork_id: &str, branch_name: &str, oid: &str) -> Result<Ref> {
        create_branch(&self.0, fork_id, branch_name, oid).await
    }

    pub async fn create_commit(
        &self,
        branch_id: &str,
        head_sha: &str,
        message: &str,
        additions: Option<Vec<create_commit::FileAddition>>,
        deletions: Option<Vec<create_commit::FileDeletion>>,
    ) -> Result<Url> {
        create_commit(&self.0, branch_id, head_sha, message, additions, deletions).await
    }

    pub async fn get_directory_content(
        &self,
        owner: &str,
        branch_name: &str,
        path: &str,
    ) -> Result<impl Iterator<Item = String> + Sized> {
        get_directory_content(&self.0, owner, WINGET_PKGS, branch_name, path).await
    }

    pub async fn get_pull_request_from_branch(
        &self,
        default_branch_name: &str,
        branch_name: &str,
    ) -> Result<Option<PullRequest>> {
        get_pull_request_from_branch(
            &self.0,
            MICROSOFT,
            WINGET_PKGS,
            default_branch_name,
            branch_name,
        )
        .await
    }

    pub async fn get_branches(&self, user: &str) -> Result<(Vec<Branch>, String)> {
        get_branches(&self.0, user, WINGET_PKGS).await
    }

    pub async fn create_pull_request(
        &self,
        repository_id: &str,
        fork_id: &str,
        fork_ref_name: &str,
        branch_name: &str,
        title: &str,
        body: &str,
    ) -> Result<Url> {
        create_pull_request(
            &self.0,
            repository_id,
            fork_id,
            fork_ref_name,
            branch_name,
            title,
            body,
        )
        .await
    }

    pub async fn delete_branch(&self, branch_id: &str) -> Result<()> {
        delete_ref(&self.0, branch_id).await
    }

    pub async fn get_all_values(
        &self,
        owner: String,
        repo: String,
        tag_name: String,
    ) -> Result<GitHubValues> {
        get_all_values(&self.0, owner, repo, tag_name).await
    }
}

pub struct Manifests {
    pub installer_manifest: InstallerManifest,
    pub default_locale_manifest: DefaultLocaleManifest,
    pub version_manifest: VersionManifest,
    pub locale_manifests: Vec<LocaleManifest>,
}
