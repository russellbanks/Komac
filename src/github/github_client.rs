use std::collections::BTreeSet;
use std::env;
use std::ops::Not;
use std::str::FromStr;

use crate::commands::cleanup::MergeState;
use crate::credential::get_default_headers;
use crate::github::graphql::create_commit::{
    CommitMessage, CommittableBranch, CreateCommit, CreateCommitOnBranchInput,
    CreateCommitVariables, FileAddition, FileChanges, FileDeletion,
};
use crate::github::graphql::create_pull_request::{
    CreatePullRequest, CreatePullRequestInput, CreatePullRequestVariables,
};
use crate::github::graphql::create_ref::{CreateRef, CreateRefVariables, Ref as CreateBranchRef};
use crate::github::graphql::get_all_values::{
    GetAllValues, GetAllValuesGitObject, GetAllValuesVariables,
};
use crate::github::graphql::get_branches::{GetBranches, PullRequest, PullRequestState};
use crate::github::graphql::get_current_user_login::GetCurrentUserLogin;
use crate::github::graphql::get_deep_directory_content::GetDeepDirectoryContent;
use crate::github::graphql::get_directory_content::{
    GetDirectoryContent, GetDirectoryContentVariables,
};
use crate::github::graphql::get_directory_content_with_text::GetDirectoryContentWithText;
use crate::github::graphql::get_existing_pull_request;
use crate::github::graphql::get_existing_pull_request::{
    GetExistingPullRequest, GetExistingPullRequestVariables,
};
use crate::github::graphql::get_repository_info::{
    GetRepositoryInfo, RepositoryVariables, TargetGitObject,
};
use crate::github::graphql::merge_upstream::{MergeUpstream, MergeUpstreamVariables};
use crate::github::graphql::types::{GitObjectId, GitRefName};
use crate::github::graphql::update_refs::{RefUpdate, UpdateRefs, UpdateRefsVariables};
use crate::github::utils::{get_package_path, is_manifest_file};
use crate::manifests::default_locale_manifest::DefaultLocaleManifest;
use crate::manifests::installer_manifest::InstallerManifest;
use crate::manifests::locale_manifest::LocaleManifest;
use crate::manifests::version_manifest::VersionManifest;
use crate::types::license::License;
use crate::types::manifest_type::ManifestType;
use crate::types::package_identifier::PackageIdentifier;
use crate::types::package_version::PackageVersion;
use crate::types::release_notes::ReleaseNotes;
use crate::types::tag::Tag;
use crate::types::urls::license_url::LicenseUrl;
use crate::types::urls::package_url::PackageUrl;
use crate::types::urls::publisher_support_url::PublisherSupportUrl;
use crate::types::urls::publisher_url::PublisherUrl;
use crate::types::urls::release_notes_url::ReleaseNotesUrl;
use color_eyre::eyre::{bail, eyre, Result, WrapErr};
use color_eyre::Report;
use const_format::{formatcp, str_repeat};
use cynic::http::ReqwestExt;
use cynic::{Id, MutationBuilder, QueryBuilder};
use indexmap::IndexMap;
use reqwest::Client;
use url::Url;

pub const MICROSOFT: &str = "microsoft";
pub const WINGET_PKGS: &str = "winget-pkgs";
pub const WINGET_PKGS_FULL_NAME: &str = formatcp!("{MICROSOFT}/{WINGET_PKGS}");
pub const GITHUB_HOST: &str = "github.com";
const GITHUB_GRAPHQL_URL: &str = "https://api.github.com/graphql";

pub struct GitHub(Client);

impl GitHub {
    pub fn new(token: &str) -> Result<Self> {
        Ok(Self(
            Client::builder()
                .default_headers(get_default_headers(Some(token)))
                .build()?,
        ))
    }

    pub async fn get_username(&self) -> Result<String> {
        const KOMAC_FORK_OWNER: &str = "KOMAC_FORK_OWNER";

        if let Ok(login) = env::var(KOMAC_FORK_OWNER) {
            Ok(login)
        } else {
            let response = self
                .0
                .post(GITHUB_GRAPHQL_URL)
                .run_graphql(GetCurrentUserLogin::build(()))
                .await?;
            response.data.map(|data| data.viewer.login).ok_or_else(|| {
                response.errors.unwrap_or_default().into_iter().fold(
                    eyre!("No data was returned when retrieving the current user's login"),
                    Report::wrap_err,
                )
            })
        }
    }

    pub async fn get_versions(
        &self,
        package_identifier: &PackageIdentifier,
    ) -> Result<BTreeSet<PackageVersion>> {
        Self::get_all_versions(
            &self.0,
            MICROSOFT,
            WINGET_PKGS,
            &get_package_path(package_identifier, None),
        )
        .await
        .wrap_err_with(|| format!("{package_identifier} does not exist in {WINGET_PKGS_FULL_NAME}"))
    }

    async fn get_all_versions(
        client: &Client,
        owner: &str,
        repo: &str,
        path: &str,
    ) -> Result<BTreeSet<PackageVersion>> {
        let response = client
            .post(GITHUB_GRAPHQL_URL)
            .run_graphql(GetDeepDirectoryContent::build(
                GetDirectoryContentVariables {
                    expression: &format!("HEAD:{path}"),
                    name: repo,
                    owner,
                },
            ))
            .await?;
        let files = response
            .data
            .and_then(|data| data.repository?.object?.into_entries())
            .ok_or_else(|| {
                response.errors.unwrap_or_default().into_iter().fold(
                    eyre!("Failed to retrieve directory content of {path}"),
                    Report::wrap_err,
                )
            })?
            .into_iter()
            .filter_map(|entry| {
                entry
                    .object?
                    .into_entries()?
                    .iter()
                    .all(|entry| entry.type_ != "tree")
                    .then(|| PackageVersion::new(&entry.name).ok())?
            })
            .collect::<BTreeSet<_>>();

        if files.is_empty() {
            bail!("No files were found for {path}")
        }

        Ok(files)
    }

    pub async fn get_manifests(
        &self,
        identifier: &PackageIdentifier,
        latest_version: &PackageVersion,
    ) -> Result<Manifests> {
        let full_package_path = get_package_path(identifier, Some(latest_version));
        let content = Self::get_directory_content_with_text(
            &self.0,
            MICROSOFT,
            WINGET_PKGS,
            &full_package_path,
        )
        .await?
        .collect::<Vec<_>>();

        let version_manifest = content
            .iter()
            .find(|file| is_manifest_file(&file.name, identifier, None, &ManifestType::Version))
            .map(|file| serde_yaml::from_str::<VersionManifest>(&file.text))
            .ok_or_else(|| eyre!("No version manifest was found in {full_package_path}"))??;

        let locale_manifests = content
            .iter()
            .filter(|file| {
                is_manifest_file(
                    &file.name,
                    identifier,
                    Some(&version_manifest.default_locale),
                    &ManifestType::Locale,
                )
            })
            .map(|file| serde_yaml::from_str::<LocaleManifest>(&file.text))
            .collect::<serde_yaml::Result<_>>()?;

        let default_locale_manifest = content
            .iter()
            .find(|file| {
                is_manifest_file(
                    &file.name,
                    identifier,
                    Some(&version_manifest.default_locale),
                    &ManifestType::DefaultLocale,
                )
            })
            .map(|file| serde_yaml::from_str::<DefaultLocaleManifest>(&file.text))
            .ok_or_else(|| {
                eyre!("No default locale manifest was found in {full_package_path}")
            })??;

        let installer_manifest = content
            .into_iter()
            .find(|file| is_manifest_file(&file.name, identifier, None, &ManifestType::Installer))
            .map(|file| serde_yaml::from_str::<InstallerManifest>(&file.text))
            .ok_or_else(|| eyre!("No installer manifest was found in {full_package_path}"))??;

        Ok(Manifests {
            installer_manifest,
            default_locale_manifest,
            version_manifest,
            locale_manifests,
        })
    }

    async fn get_directory_content_with_text(
        client: &Client,
        owner: &str,
        repo: &str,
        path: &str,
    ) -> Result<impl Iterator<Item = GitHubFile>> {
        let response = client
            .post(GITHUB_GRAPHQL_URL)
            .run_graphql(GetDirectoryContentWithText::build(
                GetDirectoryContentVariables {
                    expression: &format!("HEAD:{path}"),
                    name: repo,
                    owner,
                },
            ))
            .await?;
        response
            .data
            .and_then(|data| data.repository?.object?.into_tree_entries())
            .map(|entries| {
                entries.into_iter().filter_map(|entry| {
                    entry.object?.into_blob_text().map(|text| {
                        GitHubFile {
                            name: entry.name,
                            text
                        }
                    })
                })
            })
            .ok_or_else(|| {
                response
                    .errors
                    .unwrap_or_default()
                    .into_iter()
                    .fold(
                        eyre!("No directory content was returned when retrieving the directory content of {path}"),
                        Report::wrap_err,
                    )
            })
    }

    pub async fn get_winget_pkgs(&self, username: Option<&str>) -> Result<RepositoryData> {
        Self::get_repository_info(&self.0, username.unwrap_or(MICROSOFT), WINGET_PKGS).await
    }

    async fn get_repository_info(
        client: &Client,
        owner: &str,
        name: &str,
    ) -> Result<RepositoryData> {
        let repository = client
            .post(GITHUB_GRAPHQL_URL)
            .run_graphql(GetRepositoryInfo::build(RepositoryVariables { owner, name }))
            .await?
            .data
            .and_then(|data| data.repository)
            .ok_or_else(|| eyre!("No repository was returned when requesting repository information for {owner}/{name}"))?;

        let default_branch = repository.default_branch_ref
            .ok_or_else(|| eyre!("No default branch reference was returned when requesting repository information for {owner}/{name}"))?;

        let commits = default_branch
            .target
            .and_then(TargetGitObject::into_commit)
            .ok_or_else(|| eyre!("No default branch object was returned when requesting repository information for {owner}/{name}"))?;

        Ok(RepositoryData {
            id: repository.id,
            full_name: repository.name_with_owner,
            url: repository.url,
            default_branch_name: default_branch.name,
            default_branch_oid: commits.oid,
            default_branch_ref_id: default_branch.id,
            commit_count: commits.history.total_count,
        })
    }

    pub async fn create_branch(
        &self,
        fork_id: &Id,
        branch_name: &str,
        oid: &str,
    ) -> Result<CreateBranchRef> {
        let response = self
            .0
            .post(GITHUB_GRAPHQL_URL)
            .run_graphql(CreateRef::build(CreateRefVariables {
                name: &format!("refs/heads/{branch_name}"),
                oid: GitObjectId(oid.to_owned()),
                repository_id: fork_id,
            }))
            .await?;
        response
            .data
            .and_then(|data| data.create_ref?.ref_)
            .ok_or_else(|| {
                response.errors.unwrap_or_default().into_iter().fold(
                    eyre!("No reference was returned after creating {branch_name}"),
                    Report::wrap_err,
                )
            })
    }

    pub async fn create_commit(
        &self,
        branch_id: &Id,
        head_sha: &str,
        message: &str,
        additions: Option<Vec<FileAddition<'_>>>,
        deletions: Option<Vec<FileDeletion<'_>>>,
    ) -> Result<Url> {
        let response = self
            .0
            .post(GITHUB_GRAPHQL_URL)
            .run_graphql(CreateCommit::build(CreateCommitVariables {
                input: CreateCommitOnBranchInput {
                    branch: CommittableBranch {
                        branch_name: None,
                        id: Some(branch_id),
                        repository_name_with_owner: None,
                    },
                    client_mutation_id: None,
                    expected_head_oid: GitObjectId(head_sha.to_owned()),
                    file_changes: Some(FileChanges {
                        additions,
                        deletions,
                    }),
                    message: CommitMessage {
                        body: None,
                        headline: message,
                    },
                },
            }))
            .await?;
        response
            .data
            .and_then(|data| data.create_commit_on_branch?.commit)
            .map(|commit| commit.url)
            .ok_or_else(|| {
                response.errors.unwrap_or_default().into_iter().fold(
                    eyre!("No commit data was returned when creating commit"),
                    Report::wrap_err,
                )
            })
    }

    pub async fn get_directory_content(
        &self,
        owner: &str,
        branch_name: &str,
        path: &str,
    ) -> Result<impl Iterator<Item = String> + Sized> {
        let response = self
            .0
            .post(GITHUB_GRAPHQL_URL)
            .run_graphql(GetDirectoryContent::build(GetDirectoryContentVariables {
                expression: &format!("{branch_name}:{path}"),
                name: WINGET_PKGS,
                owner,
            }))
            .await?;
        let entries = response
            .data
            .and_then(|data| data.repository?.object?.into_entries())
            .ok_or_else(|| {
                response.errors.unwrap_or_default().into_iter().fold(
                    eyre!("No directory content was returned for {path}"),
                    Report::wrap_err,
                )
            })?;

        Ok(entries.into_iter().filter_map(|entry| entry.path))
    }

    pub async fn get_branches(
        &self,
        user: &str,
        merge_state: &MergeState,
    ) -> Result<(IndexMap<PullRequest, String>, Id)> {
        let repository = self
            .0
            .post(GITHUB_GRAPHQL_URL)
            .run_graphql(GetBranches::build(RepositoryVariables {
                owner: user,
                name: WINGET_PKGS,
            }))
            .await?
            .data
            .and_then(|data| data.repository)
            .ok_or_else(|| {
                eyre!("No repository was returned when getting branches for {user}/{WINGET_PKGS}")
            })?;

        let default_branch = repository
            .default_branch_ref
            .ok_or_else(|| eyre!("No default branch reference was returned when getting branches for {user}/{WINGET_PKGS}"))?;

        let pr_branch_map = repository
            .refs
            .map(|refs| refs.nodes)
            .ok_or_else(|| {
                eyre!("No references were returned when getting branches for {user}/{WINGET_PKGS}")
            })?
            .into_iter()
            .filter(|branch| branch.name != default_branch.name)
            .filter_map(|branch| {
                let associated_pull_requests = branch.associated_pull_requests.nodes;

                // If any associated pull request is still open, skip this branch
                if associated_pull_requests
                    .iter()
                    .any(|pull_request| pull_request.state == PullRequestState::Open)
                {
                    return None;
                }

                associated_pull_requests
                    .into_iter()
                    .filter(|pull_request| match *merge_state {
                        MergeState::MERGED => pull_request.state == PullRequestState::Merged,
                        MergeState::CLOSED => pull_request.state == PullRequestState::Closed,
                        _ => pull_request.state != PullRequestState::Open,
                    })
                    .find(|pull_request| {
                        pull_request.repository.name_with_owner == WINGET_PKGS_FULL_NAME
                    })
                    .map(|pull_request| (pull_request, branch.name))
            })
            .collect::<IndexMap<_, _>>();

        Ok((pr_branch_map, repository.id))
    }

    pub async fn create_pull_request(
        &self,
        repository_id: &Id,
        fork_id: &Id,
        fork_ref_name: &str,
        branch_name: &str,
        title: &str,
        body: &str,
    ) -> Result<Url> {
        let operation = CreatePullRequest::build(CreatePullRequestVariables {
            input: CreatePullRequestInput {
                base_ref_name: branch_name,
                body: Some(body),
                client_mutation_id: None,
                draft: None,
                head_ref_name: fork_ref_name,
                head_repository_id: Some(fork_id),
                maintainer_can_modify: None,
                repository_id,
                title,
            },
        });
        let response = self
            .0
            .post(GITHUB_GRAPHQL_URL)
            .run_graphql(operation)
            .await?;
        response
            .data
            .and_then(|data| data.create_pull_request?.pull_request)
            .map(|pull_request| pull_request.url)
            .ok_or_else(|| {
                response
                    .errors
                    .unwrap_or_default()
                    .into_iter()
                    .fold(
                        eyre!("No pull request data was returned when creating a pull request from {fork_ref_name}"),
                        Report::wrap_err,
                    )
            })
    }

    pub async fn delete_branches(&self, repository_id: &Id, branch_names: &[&str]) -> Result<()> {
        const DELETE_ID: &str = str_repeat!("0", 40);

        let response = self
            .0
            .post(GITHUB_GRAPHQL_URL)
            .run_graphql(UpdateRefs::build(UpdateRefsVariables {
                ref_updates: branch_names
                    .iter()
                    .map(|branch_name| RefUpdate {
                        after_oid: GitObjectId(DELETE_ID.to_string()),
                        before_oid: None,
                        force: None,
                        name: GitRefName(format!("refs/heads/{branch_name}")),
                    })
                    .collect(),
                repository_id,
            }))
            .await?;
        if response.data.is_some() {
            Ok(())
        } else {
            Err(response
                .errors
                .unwrap_or_default()
                .into_iter()
                .fold(eyre!("Failed to delete branch refs"), Report::wrap_err))
        }
    }

    pub async fn get_existing_pull_request(
        &self,
        identifier: &PackageIdentifier,
        version: &PackageVersion,
    ) -> Result<Option<get_existing_pull_request::PullRequest>> {
        let response = self
            .0
            .post(GITHUB_GRAPHQL_URL)
            .run_graphql(GetExistingPullRequest::build(GetExistingPullRequestVariables {
                query: &format!("repo:{WINGET_PKGS_FULL_NAME} is:pull-request in:title {identifier} {version}"),
            }))
            .await?;

        Ok(response.data.and_then(|data| {
            data.search
                .edges
                .into_iter()
                .next()?
                .node?
                .into_pull_request()
        }))
    }

    pub async fn get_all_values(
        &self,
        owner: String,
        repo: String,
        tag_name: String,
    ) -> Result<GitHubValues> {
        let data = self
            .0
            .post(GITHUB_GRAPHQL_URL)
            .run_graphql(GetAllValues::build(GetAllValuesVariables {
                name: &repo,
                owner: &owner,
                tag_name: &tag_name,
            }))
            .await?
            .data
            .ok_or_else(|| eyre!("No data was returned when parsing values from {owner}/{repo}"))?;

        let repository = data.repository.ok_or_else(|| {
            eyre!("No repository was returned when parsing values from {owner}/{repo}")
        })?;

        let object = repository.object.ok_or_else(|| {
            eyre!("No directory content was returned when getting root directory content for {owner}/{repo}")
        })?;

        let license_url = match object {
            GetAllValuesGitObject::Tree(tree) => tree
                .entries
                .into_iter()
                .filter_map(|entry| (entry.type_ == "blob").then_some(entry.name))
                .find(|name| {
                    name.rfind('.').map_or_else(
                        || name.to_ascii_lowercase(),
                        |dot_index| name[..dot_index].to_ascii_lowercase(),
                    ) == "license"
                })
                .map(|name| {
                    LicenseUrl::from_str(&format!(
                        "https://github.com/{owner}/{repo}/blob/HEAD/{name}"
                    ))
                }),
            GetAllValuesGitObject::Unknown => None,
        }
        .transpose()?;

        let release = repository.release;

        let topics = repository
            .repository_topics
            .nodes
            .into_iter()
            .filter_map(|topic_node| Tag::try_new(topic_node.topic.name).ok())
            .collect::<BTreeSet<_>>();

        let publisher_support_url = repository
            .has_issues_enabled
            .then(|| {
                PublisherSupportUrl::from_str(&format!("https://github.com/{owner}/{repo}/issues"))
            })
            .transpose()?;

        Ok(GitHubValues {
            publisher_url: PublisherUrl::from_str(repository.owner.url.as_str())?,
            publisher_support_url,
            short_description: repository.description.unwrap_or_default(),
            license: repository
                .license_info
                .and_then(|mut info| {
                    info.pseudo_license
                        .not()
                        .then_some(info.spdx_id.unwrap_or_else(|| {
                            info.key.make_ascii_uppercase();
                            info.key
                        }))
                })
                .and_then(|license| License::try_new(license).ok()),
            license_url,
            package_url: PackageUrl::from_str(repository.url.as_str())?,
            release_notes: release.as_ref().and_then(|release| {
                ReleaseNotes::format(release.description.as_ref()?, &owner, &repo)
            }),
            release_notes_url: release
                .and_then(|release| ReleaseNotesUrl::from_str(release.url.as_str()).ok()),
            topics: Option::from(topics).filter(|topics| !topics.is_empty()),
        })
    }

    pub async fn merge_upstream(
        &self,
        branch_ref_id: &Id,
        upstream_target_oid: GitObjectId,
        force: bool,
    ) -> Result<()> {
        let response = self
            .0
            .post(GITHUB_GRAPHQL_URL)
            .run_graphql(MergeUpstream::build(MergeUpstreamVariables {
                branch_ref_id,
                upstream_target_oid,
                force,
            }))
            .await?;
        if response.data.is_some() {
            Ok(())
        } else {
            Err(response.errors.unwrap_or_default().into_iter().fold(
                eyre!("Failed to merge upstream branch into fork branch"),
                Report::wrap_err,
            ))
        }
    }
}

pub struct Manifests {
    pub installer_manifest: InstallerManifest,
    pub default_locale_manifest: DefaultLocaleManifest,
    pub version_manifest: VersionManifest,
    pub locale_manifests: Vec<LocaleManifest>,
}

pub struct GitHubValues {
    pub publisher_url: PublisherUrl,
    pub publisher_support_url: Option<PublisherSupportUrl>,
    pub short_description: String,
    pub license: Option<License>,
    pub license_url: Option<LicenseUrl>,
    pub package_url: PackageUrl,
    pub release_notes: Option<ReleaseNotes>,
    pub release_notes_url: Option<ReleaseNotesUrl>,
    pub topics: Option<BTreeSet<Tag>>,
}

pub struct GitHubFile {
    pub name: String,
    pub text: String,
}

pub struct RepositoryData {
    pub id: Id,
    pub full_name: String,
    pub url: Url,
    pub default_branch_name: String,
    pub default_branch_oid: GitObjectId,
    pub default_branch_ref_id: Id,
    pub commit_count: i32,
}
