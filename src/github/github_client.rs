use crate::commands::cleanup::MergeState;
use crate::commands::utils::SPINNER_TICK_RATE;
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
    GetAllValues, GetAllValuesGitObject, GetAllValuesVariables, Tree,
};
use crate::github::graphql::get_branches::{
    GetBranches, GetBranchesVariables, PullRequest, PullRequestState, RefConnection,
};
use crate::github::graphql::get_current_user_login::GetCurrentUserLogin;
use crate::github::graphql::get_directory_content::{
    GetDirectoryContent, GetDirectoryContentVariables,
};
use crate::github::graphql::get_directory_content_with_text::{
    GetDirectoryContentWithText, TreeEntry,
};
use crate::github::graphql::get_existing_pull_request;
use crate::github::graphql::get_existing_pull_request::{
    GetExistingPullRequest, GetExistingPullRequestVariables,
};
use crate::github::graphql::get_file_content::GetFileContent;
use crate::github::graphql::get_repository_info::{
    GetRepositoryInfo, RepositoryVariables, TargetGitObject,
};
use crate::github::graphql::merge_upstream::{MergeUpstream, MergeUpstreamVariables};
use crate::github::graphql::types::{Base64String, GitObjectId, GitRefName};
use crate::github::graphql::update_refs::{RefUpdate, UpdateRefs, UpdateRefsVariables};
use crate::github::rest::get_tree::GitTree;
use crate::github::rest::GITHUB_JSON_MIME;
use crate::github::utils::{
    get_branch_name, get_commit_title, get_package_path, is_manifest_file, pull_request_body,
};
use crate::manifests::default_locale_manifest::DefaultLocaleManifest;
use crate::manifests::installer_manifest::InstallerManifest;
use crate::manifests::locale_manifest::LocaleManifest;
use crate::manifests::version_manifest::VersionManifest;
use crate::manifests::{ManifestTrait, Manifests};
use crate::types::license::License;
use crate::types::manifest_type::{ManifestType, ManifestTypeWithLocale};
use crate::types::package_identifier::PackageIdentifier;
use crate::types::package_version::PackageVersion;
use crate::types::release_notes::ReleaseNotes;
use crate::types::tag::Tag;
use crate::types::urls::license_url::LicenseUrl;
use crate::types::urls::package_url::PackageUrl;
use crate::types::urls::publisher_support_url::PublisherSupportUrl;
use crate::types::urls::publisher_url::PublisherUrl;
use crate::types::urls::release_notes_url::ReleaseNotesUrl;
use crate::types::urls::url::DecodedUrl;
use crate::update_state::UpdateState;
use base64ct::{Base64, Encoding};
use bon::bon;
use const_format::{formatcp, str_repeat};
use cynic::http::{CynicReqwestError, ReqwestExt};
use cynic::{GraphQlError, GraphQlResponse, Id, MutationBuilder, QueryBuilder};
use indexmap::IndexMap;
use indicatif::ProgressBar;
use itertools::Itertools;
use owo_colors::OwoColorize;
use reqwest::header::ACCEPT;
use reqwest::Client;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::env;
use std::future::Future;
use std::num::NonZeroU32;
use std::ops::Not;
use std::str::FromStr;
use thiserror::Error;
use url::Url;

pub const MICROSOFT: &str = "microsoft";
pub const WINGET_PKGS: &str = "winget-pkgs";
pub const WINGET_PKGS_FULL_NAME: &str = formatcp!("{MICROSOFT}/{WINGET_PKGS}");
pub const GITHUB_HOST: &str = "github.com";
const GITHUB_GRAPHQL_URL: &str = "https://api.github.com/graphql";

#[derive(Debug, Error)]
pub enum GitHubError {
    #[error("{}", .0.clone().unwrap_or_default().into_iter().next().map_or_else(|| String::from("Unknown GraphQL error"), |err| err.message))]
    GraphQL(Option<Vec<GraphQlError>>),
    #[error("{0} does not exist in {WINGET_PKGS_FULL_NAME}")]
    PackageNonExistent(PackageIdentifier),
    #[error("No {type} manifest was found in {path}")]
    ManifestNotFound { r#type: ManifestType, path: String },
    #[error("No valid files were found for {path}")]
    NoValidFiles { path: String },
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    CynicRequest(#[from] CynicReqwestError),
    #[error(transparent)]
    YamlError(#[from] serde_yaml::Error),
    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),
}

pub struct GitHub(Client);

#[bon]
impl GitHub {
    pub fn new(token: &str) -> Result<Self, GitHubError> {
        Ok(Self(
            Client::builder()
                .default_headers(get_default_headers(Some(token)))
                .build()?,
        ))
    }

    pub async fn get_username(&self) -> Result<String, GitHubError> {
        const KOMAC_FORK_OWNER: &str = "KOMAC_FORK_OWNER";

        if let Ok(login) = env::var(KOMAC_FORK_OWNER) {
            Ok(login)
        } else {
            let GraphQlResponse { data, errors } = self
                .0
                .post(GITHUB_GRAPHQL_URL)
                .run_graphql(GetCurrentUserLogin::build(()))
                .await?;
            data.map(|data| data.viewer.login)
                .ok_or(GitHubError::GraphQL(errors))
        }
    }

    pub async fn get_versions(
        &self,
        package_identifier: &PackageIdentifier,
    ) -> Result<BTreeSet<PackageVersion>, GitHubError> {
        self.get_all_versions(
            MICROSOFT,
            WINGET_PKGS,
            &get_package_path(package_identifier, None, None),
        )
        .await
        .map_err(|_| GitHubError::PackageNonExistent(package_identifier.clone()))
    }

    async fn get_all_versions(
        &self,
        owner: &str,
        repo: &str,
        path: &str,
    ) -> Result<BTreeSet<PackageVersion>, GitHubError> {
        const TREE: &str = "tree";
        const SEPARATOR: char = '/';

        let endpoint = format!(
            "https://api.github.com/repos/{owner}/{repo}/git/trees/HEAD:{path}?recursive=true"
        );

        let GitTree { tree, .. } = self
            .0
            .get(endpoint)
            .header(ACCEPT, GITHUB_JSON_MIME)
            .send()
            .await?
            .json::<GitTree>()
            .await?;

        let files = tree
            .iter()
            .filter(|entry| entry.path.matches(SEPARATOR).count() == 1)
            .chunk_by(|entry| {
                entry
                    .path
                    .split_once(SEPARATOR)
                    .map_or(entry.path.as_str(), |(version, _rest)| version)
            })
            .into_iter()
            .filter_map(|(version, mut group)| {
                group
                    .all(|object| object.r#type != TREE)
                    .then(|| PackageVersion::from_str(version).ok())?
            })
            .collect::<BTreeSet<_>>();

        Option::from(files)
            .filter(|files| !files.is_empty())
            .ok_or_else(|| GitHubError::NoValidFiles {
                path: path.to_owned(),
            })
    }

    pub async fn get_manifests(
        &self,
        identifier: &PackageIdentifier,
        latest_version: &PackageVersion,
    ) -> Result<Manifests, GitHubError> {
        let full_package_path = get_package_path(identifier, Some(latest_version), None);
        let content = self
            .get_directory_content_with_text(MICROSOFT, WINGET_PKGS, &full_package_path)
            .await?
            .collect::<Vec<_>>();

        let version_manifest = content
            .iter()
            .find(|file| is_manifest_file::<VersionManifest>(&file.name, identifier, None))
            .map(|file| serde_yaml::from_str::<VersionManifest>(&file.text))
            .ok_or_else(|| GitHubError::ManifestNotFound {
                r#type: ManifestType::Version,
                path: full_package_path.clone(),
            })??;

        let locale_manifests = content
            .iter()
            .filter(|file| {
                is_manifest_file::<LocaleManifest>(
                    &file.name,
                    identifier,
                    Some(&version_manifest.default_locale),
                )
            })
            .map(|file| serde_yaml::from_str::<LocaleManifest>(&file.text))
            .collect::<serde_yaml::Result<_>>()?;

        let default_locale_manifest = content
            .iter()
            .find(|file| {
                is_manifest_file::<DefaultLocaleManifest>(
                    &file.name,
                    identifier,
                    Some(&version_manifest.default_locale),
                )
            })
            .map(|file| serde_yaml::from_str::<DefaultLocaleManifest>(&file.text))
            .ok_or_else(|| GitHubError::ManifestNotFound {
                r#type: ManifestType::DefaultLocale,
                path: full_package_path.clone(),
            })??;

        let installer_manifest = content
            .into_iter()
            .find(|file| is_manifest_file::<InstallerManifest>(&file.name, identifier, None))
            .map(|file| serde_yaml::from_str::<InstallerManifest>(&file.text))
            .ok_or_else(|| GitHubError::ManifestNotFound {
                r#type: ManifestType::Installer,
                path: full_package_path.clone(),
            })??;

        Ok(Manifests {
            installer: installer_manifest,
            default_locale: default_locale_manifest,
            locales: locale_manifests,
            version: version_manifest,
        })
    }

    async fn get_directory_content_with_text(
        &self,
        owner: &str,
        repo: &str,
        path: &str,
    ) -> Result<impl Iterator<Item = GitHubFile>, GitHubError> {
        let GraphQlResponse { data, errors } = self
            .0
            .post(GITHUB_GRAPHQL_URL)
            .run_graphql(GetDirectoryContentWithText::build(
                GetDirectoryContentVariables {
                    owner,
                    name: repo,
                    expression: &format!("HEAD:{path}"),
                },
            ))
            .await?;
        data.and_then(|data| data.repository?.object?.into_tree_entries())
            .map(|entries| {
                entries
                    .into_iter()
                    .filter_map(|TreeEntry { name, object }| {
                        object?
                            .into_blob_text()
                            .map(|text| GitHubFile { name, text })
                    })
            })
            .ok_or(GitHubError::GraphQL(errors))
    }

    pub async fn get_manifest<T: ManifestTrait + for<'de> Deserialize<'de>>(
        &self,
        identifier: &PackageIdentifier,
        version: &PackageVersion,
        manifest_type: ManifestTypeWithLocale,
    ) -> Result<T, GitHubError> {
        let path = get_package_path(identifier, Some(version), Some(&manifest_type));
        let content = self.get_file_content(MICROSOFT, WINGET_PKGS, &path).await?;
        let manifest = serde_yaml::from_str::<T>(&content)?;
        Ok(manifest)
    }

    async fn get_file_content(
        &self,
        owner: &str,
        repo: &str,
        path: &str,
    ) -> Result<String, GitHubError> {
        let GraphQlResponse { data, errors } = self
            .0
            .post(GITHUB_GRAPHQL_URL)
            .run_graphql(GetFileContent::build(GetDirectoryContentVariables {
                owner,
                name: repo,
                expression: &format!("HEAD:{path}"),
            }))
            .await?;
        data.and_then(|data| data.repository?.object?.into_blob_text())
            .ok_or(GitHubError::GraphQL(errors))
    }

    #[builder(finish_fn = send)]
    pub async fn get_winget_pkgs(
        &self,
        owner: Option<&str>,
    ) -> Result<RepositoryData, GitHubError> {
        self.get_repository_info(owner.unwrap_or(MICROSOFT), WINGET_PKGS)
            .await
    }

    async fn get_repository_info(
        &self,
        owner: &str,
        name: &str,
    ) -> Result<RepositoryData, GitHubError> {
        let GraphQlResponse { data, errors } = self
            .0
            .post(GITHUB_GRAPHQL_URL)
            .run_graphql(GetRepositoryInfo::build(RepositoryVariables {
                owner,
                name,
            }))
            .await?;

        let repository = data
            .and_then(|data| data.repository)
            .ok_or_else(|| GitHubError::GraphQL(errors.clone()))?;

        let default_branch = repository
            .default_branch_ref
            .ok_or_else(|| GitHubError::GraphQL(errors.clone()))?;

        let commits = default_branch
            .target
            .and_then(TargetGitObject::into_commit)
            .ok_or(GitHubError::GraphQL(errors))?;

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
        oid: GitObjectId,
    ) -> Result<CreateBranchRef, GitHubError> {
        let GraphQlResponse { data, errors } = self
            .0
            .post(GITHUB_GRAPHQL_URL)
            .run_graphql(CreateRef::build(CreateRefVariables {
                name: &format!("refs/heads/{branch_name}"),
                oid,
                repository_id: fork_id,
            }))
            .await?;
        data.and_then(|data| data.create_ref?.ref_)
            .ok_or(GitHubError::GraphQL(errors))
    }

    #[builder(finish_fn = send)]
    pub async fn create_commit(
        &self,
        branch_id: &Id,
        head_sha: GitObjectId,
        message: &str,
        additions: Option<Vec<FileAddition<'_>>>,
        deletions: Option<Vec<FileDeletion<'_>>>,
    ) -> Result<Url, GitHubError> {
        let GraphQlResponse { data, errors } = self
            .0
            .post(GITHUB_GRAPHQL_URL)
            .run_graphql(CreateCommit::build(CreateCommitVariables {
                input: CreateCommitOnBranchInput {
                    branch: CommittableBranch { id: branch_id },
                    expected_head_oid: head_sha,
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
        data.and_then(|data| data.create_commit_on_branch?.commit)
            .map(|commit| commit.url)
            .ok_or(GitHubError::GraphQL(errors))
    }

    pub async fn get_directory_content(
        &self,
        owner: &str,
        branch_name: &str,
        path: &str,
    ) -> Result<impl Iterator<Item = String> + Sized, GitHubError> {
        let GraphQlResponse { data, errors } = self
            .0
            .post(GITHUB_GRAPHQL_URL)
            .run_graphql(GetDirectoryContent::build(GetDirectoryContentVariables {
                expression: &format!("{branch_name}:{path}"),
                name: WINGET_PKGS,
                owner,
            }))
            .await?;
        let entries = data
            .and_then(|data| data.repository?.object?.into_entries())
            .ok_or(GitHubError::GraphQL(errors))?;

        Ok(entries.into_iter().filter_map(|entry| entry.path))
    }

    pub async fn get_branches(
        &self,
        user: &str,
        merge_state: MergeState,
    ) -> Result<(IndexMap<PullRequest, String>, Id), GitHubError> {
        let mut pr_branch_map = IndexMap::new();
        let mut cursor = None;

        loop {
            let GraphQlResponse { data, errors } = self
                .0
                .post(GITHUB_GRAPHQL_URL)
                .run_graphql(GetBranches::build(GetBranchesVariables {
                    owner: user,
                    name: WINGET_PKGS,
                    cursor: cursor.as_deref(),
                }))
                .await?;

            let repository = data
                .and_then(|data| data.repository)
                .ok_or_else(|| GitHubError::GraphQL(errors.clone()))?;

            let default_branch = repository
                .default_branch_ref
                .ok_or_else(|| GitHubError::GraphQL(errors.clone()))?;

            let RefConnection {
                branches,
                page_info,
            } = repository.refs.ok_or(GitHubError::GraphQL(errors))?;

            for branch in branches
                .into_iter()
                .filter(|branch| branch.name != default_branch.name)
                .filter(|branch| {
                    branch
                        .associated_pull_requests
                        .pull_requests
                        .iter()
                        .all(|pull_request| pull_request.state != PullRequestState::Open)
                })
            {
                if let Some(pull_request) = branch
                    .associated_pull_requests
                    .pull_requests
                    .into_iter()
                    .filter(|pull_request| match merge_state {
                        MergeState::MERGED => pull_request.state == PullRequestState::Merged,
                        MergeState::CLOSED => pull_request.state == PullRequestState::Closed,
                        _ => pull_request.state != PullRequestState::Open,
                    })
                    .find(|pull_request| {
                        pull_request.repository.name_with_owner == WINGET_PKGS_FULL_NAME
                    })
                {
                    pr_branch_map.insert(pull_request, branch.name);
                }
            }

            if page_info.has_next_page {
                cursor = page_info.end_cursor;
            } else {
                return Ok((pr_branch_map, repository.id));
            }
        }
    }

    pub async fn create_pull_request(
        &self,
        repository_id: &Id,
        fork_id: &Id,
        fork_ref_name: &str,
        branch_name: &str,
        title: &str,
        body: &str,
    ) -> Result<Url, GitHubError> {
        let operation = CreatePullRequest::build(CreatePullRequestVariables {
            input: CreatePullRequestInput {
                base_ref_name: branch_name,
                body: Some(body),
                draft: None,
                head_ref_name: fork_ref_name,
                head_repository_id: Some(fork_id),
                maintainer_can_modify: None,
                repository_id,
                title,
            },
        });
        let GraphQlResponse { data, errors } = self
            .0
            .post(GITHUB_GRAPHQL_URL)
            .run_graphql(operation)
            .await?;
        data.and_then(|data| data.create_pull_request?.pull_request)
            .map(|pull_request| pull_request.url)
            .ok_or(GitHubError::GraphQL(errors))
    }

    pub async fn delete_branches(
        &self,
        repository_id: &Id,
        branch_names: &[&str],
    ) -> Result<(), GitHubError> {
        const DELETE_ID: &str = str_repeat!("0", 40);

        let GraphQlResponse { data, errors } = self
            .0
            .post(GITHUB_GRAPHQL_URL)
            .run_graphql(UpdateRefs::build(UpdateRefsVariables {
                ref_updates: branch_names
                    .iter()
                    .map(|branch_name| RefUpdate {
                        after_oid: GitObjectId::new(DELETE_ID),
                        before_oid: None,
                        force: None,
                        name: GitRefName::new(format!("refs/heads/{branch_name}")),
                    })
                    .collect(),
                repository_id,
            }))
            .await?;
        if data.is_some() {
            Ok(())
        } else {
            Err(GitHubError::GraphQL(errors))
        }
    }

    pub async fn get_existing_pull_request(
        &self,
        identifier: &PackageIdentifier,
        version: &PackageVersion,
    ) -> Result<Option<get_existing_pull_request::PullRequest>, GitHubError> {
        self
            .0
            .post(GITHUB_GRAPHQL_URL)
            .run_graphql(GetExistingPullRequest::build(GetExistingPullRequestVariables {
                query: &format!("repo:{WINGET_PKGS_FULL_NAME} is:pull-request in:title {identifier} {version}"),
            }))
            .await
            .map(|response| response.data.and_then(|data| {
                data.search
                    .edges
                    .into_iter()
                    .next()?
                    .node?
                    .into_pull_request()
            }))
            .map_err(GitHubError::CynicRequest)
    }

    pub fn get_all_values_from_url(
        &self,
        url: &DecodedUrl,
    ) -> Option<impl Future<Output = Result<GitHubValues, GitHubError>> + use<'_>> {
        let mut parts = url.path_segments()?;
        let _file_name = parts.next_back()?;
        let builder = self
            .get_all_values()
            .owner(parts.next()?.to_owned())
            .repo(parts.next()?.to_owned());
        let _releases = parts.next()?;
        let _download = parts.next()?;
        Some(builder.tag_name(parts.join("/")).send())
    }

    #[builder(finish_fn = send)]
    pub async fn get_all_values(
        &self,
        owner: String,
        repo: String,
        tag_name: String,
    ) -> Result<GitHubValues, GitHubError> {
        let GraphQlResponse { data, errors } = self
            .0
            .post(GITHUB_GRAPHQL_URL)
            .run_graphql(GetAllValues::build(GetAllValuesVariables {
                name: &repo,
                owner: &owner,
                tag_name: &tag_name,
            }))
            .await?;

        let values = data.ok_or_else(|| GitHubError::GraphQL(errors.clone()))?;

        let repository = values
            .repository
            .ok_or_else(|| GitHubError::GraphQL(errors.clone()))?;

        let object = repository.object.ok_or(GitHubError::GraphQL(errors))?;

        let license_url = match object {
            GetAllValuesGitObject::Tree(Tree { entries }) => entries
                .into_iter()
                .filter_map(|entry| (entry.type_ == "blob").then_some(entry.name))
                .find(|name| {
                    name.rfind('.').map_or_else(
                        || name.to_ascii_lowercase(),
                        |dot_index| name[..dot_index].to_ascii_lowercase(),
                    ) == "license"
                })
                .and_then(|name| {
                    LicenseUrl::from_str(&format!(
                        "https://github.com/{owner}/{repo}/blob/HEAD/{name}"
                    ))
                    .ok()
                }),
            GetAllValuesGitObject::Unknown => None,
        };

        let release = repository.release;

        let topics = repository
            .topics
            .nodes
            .into_iter()
            .flat_map(|topic_node| Tag::try_new(topic_node.topic.name))
            .collect::<BTreeSet<_>>();

        let publisher_support_url = if repository.has_issues_enabled {
            PublisherSupportUrl::from_str(&format!("https://github.com/{owner}/{repo}/issues")).ok()
        } else {
            None
        };

        Ok(GitHubValues {
            publisher_url: PublisherUrl::from_str(repository.owner.url.as_str())?,
            publisher_support_url,
            license: repository
                .license_info
                .and_then(|mut license| {
                    license.is_pseudo.not().then(|| {
                        license.spdx_id.unwrap_or_else(|| {
                            license.key.make_ascii_uppercase();
                            license.key
                        })
                    })
                })
                .and_then(|license| License::try_new(license).ok()),
            license_url,
            package_url: PackageUrl::from_str(repository.url.as_str())?,
            release_notes: release
                .as_ref()
                .and_then(|release| ReleaseNotes::format(release.description_html.as_deref()?)),
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
    ) -> Result<(), GitHubError> {
        let GraphQlResponse { data, errors } = self
            .0
            .post(GITHUB_GRAPHQL_URL)
            .run_graphql(MergeUpstream::build(MergeUpstreamVariables {
                branch_ref_id,
                upstream_target_oid,
                force,
            }))
            .await?;
        if data.is_some() {
            Ok(())
        } else {
            Err(GitHubError::GraphQL(errors))
        }
    }

    #[builder(finish_fn = send)]
    pub async fn remove_version(
        &self,
        identifier: &PackageIdentifier,
        version: &PackageVersion,
        reason: String,
        fork_owner: &String,
        fork: &RepositoryData,
        winget_pkgs: &RepositoryData,
        issue_resolves: Option<Vec<NonZeroU32>>,
    ) -> Result<Url, GitHubError> {
        // Create an indeterminate progress bar to show as a pull request is being created
        let pr_progress = ProgressBar::new_spinner().with_message(format!(
            "Creating a pull request to remove {identifier} {version}",
        ));
        pr_progress.enable_steady_tick(SPINNER_TICK_RATE);

        let branch_name = get_branch_name(identifier, version);
        let pull_request_branch = self
            .create_branch(
                &fork.id,
                &branch_name,
                winget_pkgs.default_branch_oid.clone(),
            )
            .await?;
        let commit_title = get_commit_title(identifier, version, &UpdateState::RemoveVersion);
        let deletions = self
            .get_directory_content(
                fork_owner,
                &branch_name,
                &get_package_path(identifier, Some(version), None),
            )
            .await?
            .map(FileDeletion::new)
            .collect::<Vec<_>>();
        let _commit_url = self
            .create_commit()
            .branch_id(&pull_request_branch.id)
            .head_sha(pull_request_branch.target.map(|object| object.oid).unwrap())
            .message(&commit_title)
            .deletions(deletions)
            .send()
            .await?;
        let pull_request_url = self
            .create_pull_request(
                &winget_pkgs.id,
                &fork.id,
                &format!("{fork_owner}:{}", pull_request_branch.name),
                &winget_pkgs.default_branch_name,
                &commit_title,
                &pull_request_body()
                    .maybe_issue_resolves(issue_resolves)
                    .alternative_text(reason)
                    .get(),
            )
            .await?;

        pr_progress.finish_and_clear();

        println!(
            "{} created a pull request to remove {identifier} {version}",
            "Successfully".green(),
        );
        println!("{}", pull_request_url.as_str());
        Ok(pull_request_url)
    }

    #[builder(finish_fn = send)]
    pub async fn add_version(
        &self,
        identifier: &PackageIdentifier,
        version: &PackageVersion,
        versions: Option<&BTreeSet<PackageVersion>>,
        changes: Vec<(String, String)>,
        replace_version: Option<&PackageVersion>,
        issue_resolves: Option<Vec<NonZeroU32>>,
        created_with: Option<String>,
        created_with_url: Option<DecodedUrl>,
    ) -> Result<Url, GitHubError> {
        let current_user = self.get_username();
        let winget_pkgs = self.get_winget_pkgs().send().await?;
        let current_user = current_user.await?;
        let fork = self.get_winget_pkgs().owner(&current_user).send().await?;
        let branch_name = get_branch_name(identifier, version);
        let pull_request_branch = self
            .create_branch(&fork.id, &branch_name, winget_pkgs.default_branch_oid)
            .await?;
        let commit_title =
            get_commit_title(identifier, version, &UpdateState::get(version, versions));
        let additions = changes
            .iter()
            .map(|(path, content)| {
                FileAddition::new(
                    Base64String::new(Base64::encode_string(content.as_bytes())),
                    path,
                )
            })
            .collect::<Vec<_>>();
        let deletions = if replace_version.is_some() {
            Some(
                self.get_directory_content(
                    &current_user,
                    &branch_name,
                    &get_package_path(identifier, replace_version, None),
                )
                .await?
                .map(FileDeletion::new)
                .collect::<Vec<_>>(),
            )
        } else {
            None
        };
        let _commit_url = self
            .create_commit()
            .branch_id(&pull_request_branch.id)
            .head_sha(pull_request_branch.target.map(|target| target.oid).unwrap())
            .message(&commit_title)
            .additions(additions)
            .maybe_deletions(deletions)
            .send()
            .await?;
        self.create_pull_request(
            &winget_pkgs.id,
            &fork.id,
            &format!("{current_user}:{}", pull_request_branch.name),
            &winget_pkgs.default_branch_name,
            &commit_title,
            &pull_request_body()
                .maybe_issue_resolves(issue_resolves)
                .maybe_created_with(created_with)
                .maybe_created_with_url(created_with_url)
                .get(),
        )
        .await
    }
}

pub struct GitHubValues {
    pub publisher_url: PublisherUrl,
    pub publisher_support_url: Option<PublisherSupportUrl>,
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
