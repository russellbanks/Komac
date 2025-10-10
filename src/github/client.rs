use std::{borrow::Cow, collections::BTreeSet, num::NonZeroU32, str::FromStr};

use bon::bon;
use color_eyre::eyre::eyre;
use cynic::{GraphQlResponse, Id, MutationBuilder, QueryBuilder, http::ReqwestExt};
use futures_util::future::OptionFuture;
use indexmap::IndexMap;
use indicatif::ProgressBar;
use itertools::Itertools;
use owo_colors::OwoColorize;
use reqwest::Client;
use serde::de::DeserializeOwned;
use url::Url;
use winget_types::{
    Manifest, ManifestType, ManifestTypeWithLocale, PackageIdentifier, PackageVersion,
    installer::InstallerManifest,
    locale::{DefaultLocaleManifest, License, LocaleManifest, ReleaseNotes, Tag},
    url::{DecodedUrl, LicenseUrl, PackageUrl, PublisherSupportUrl, PublisherUrl, ReleaseNotesUrl},
    version::VersionManifest,
};

use super::GitHubError;
use crate::{
    commands::{cleanup::MergeState, utils::SPINNER_TICK_RATE},
    github::{
        MICROSOFT, WINGET_PKGS, WINGET_PKGS_FULL_NAME,
        graphql::{
            GRAPHQL_URL,
            create_commit::{FileAddition, FileDeletion},
            create_pull_request::{
                CreatePullRequest, CreatePullRequestInput, CreatePullRequestVariables,
            },
            create_ref::{CreateRef, CreateRefVariables, Ref as CreateBranchRef},
            get_all_values::{GetAllValues, GetAllValuesGitObject, GetAllValuesVariables, Tree},
            get_branches::{GetBranches, GetBranchesVariables, PullRequest, RefConnection},
            get_directory_content::GetDirectoryContentVariables,
            get_directory_content_with_text::{GetDirectoryContentWithText, TreeEntry},
            get_repository_info::{GetRepositoryInfo, RepositoryVariables, TargetGitObject},
            types::GitObjectId,
            update_refs::{RefUpdate, UpdateRefs, UpdateRefsInput},
        },
        utils::{
            CommitTitle, PackagePath, branch_name, commit_title, is_manifest_file,
            pull_request_body,
        },
    },
    manifests::Manifests,
    token::default_headers,
    traits::FromHtml,
    update_state::UpdateState,
};

#[derive(Clone)]
#[repr(transparent)]
pub struct GitHub(pub(super) Client);

#[bon]
impl GitHub {
    pub fn new<T: AsRef<str>>(token: T) -> Result<Self, GitHubError> {
        Ok(Self(
            Client::builder()
                .default_headers(default_headers(Some(token.as_ref())))
                .build()?,
        ))
    }

    pub async fn get_manifests(
        &self,
        identifier: &PackageIdentifier,
        latest_version: &PackageVersion,
    ) -> Result<Manifests, GitHubError> {
        let full_package_path = PackagePath::new(identifier, Some(latest_version), None);
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
        path: &PackagePath,
    ) -> Result<impl Iterator<Item = GitHubFile>, GitHubError> {
        let GraphQlResponse { data, errors } = self
            .0
            .post(GRAPHQL_URL)
            .run_graphql(GetDirectoryContentWithText::build(
                GetDirectoryContentVariables::new(&owner, &repo, &format!("HEAD:{path}")),
            ))
            .await?;

        data.and_then(|data| data.repository?.object?.into_tree_entries())
            .map(|entries| {
                entries
                    .into_iter()
                    .filter_map(|TreeEntry { name, object }| {
                        object?
                            .into_blob_text()
                            .map(|text| GitHubFile::new(name, text))
                    })
            })
            .ok_or_else(|| {
                GitHubError::graphql_errors(eyre!("failed to get directory content"), errors)
            })
    }

    pub async fn get_manifest<T: Manifest + DeserializeOwned>(
        &self,
        identifier: &PackageIdentifier,
        version: &PackageVersion,
        manifest_type: ManifestTypeWithLocale,
    ) -> Result<T, GitHubError> {
        let path = PackagePath::new(identifier, Some(version), Some(&manifest_type));
        let content = self.get_file_content(MICROSOFT, WINGET_PKGS, &path).await?;
        let manifest = serde_yaml::from_str::<T>(&content)?;
        Ok(manifest)
    }

    #[builder(finish_fn = send)]
    pub async fn get_winget_pkgs(
        &self,
        #[builder(into)] owner: Option<Cow<'_, str>>,
    ) -> Result<RepositoryData, GitHubError> {
        self.get_repository_info(owner.as_deref().unwrap_or(MICROSOFT), WINGET_PKGS)
            .await
    }

    async fn get_repository_info(
        &self,
        owner: &str,
        name: &str,
    ) -> Result<RepositoryData, GitHubError> {
        let GraphQlResponse { data, errors } = self
            .0
            .post(GRAPHQL_URL)
            .run_graphql(GetRepositoryInfo::build(RepositoryVariables::new(
                owner, name,
            )))
            .await?;

        let repository = data.and_then(|data| data.repository).ok_or_else(|| {
            GitHubError::graphql_errors(
                eyre!("failed to get {owner}/{name} when getting repository info"),
                errors.clone(),
            )
        })?;

        let default_branch = repository.default_branch_ref.ok_or_else(|| {
            GitHubError::graphql_errors(
                eyre!("failed to get default branch ref from {owner}/{name} when getting repository info"),
                errors.clone(),
            )
        })?;

        let commits = default_branch
            .target
            .and_then(TargetGitObject::into_commit)
            .ok_or_else(|| {
                GitHubError::graphql_errors(
                    eyre!("failed to get commits from {owner}/{name} when getting repository info"),
                    errors,
                )
            })?;

        Ok(RepositoryData {
            id: repository.id,
            owner: repository.owner.login,
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
            .post(GRAPHQL_URL)
            .run_graphql(CreateRef::build(
                CreateRefVariables::builder()
                    .name(&format!("refs/heads/{branch_name}"))
                    .oid(oid)
                    .repository_id(fork_id)
                    .build(),
            ))
            .await?;

        data.and_then(|data| data.create_ref?.ref_).ok_or_else(|| {
            GitHubError::graphql_errors(eyre!("failed to create branch {branch_name}"), errors)
        })
    }

    pub async fn get_branches<T: AsRef<str>>(
        &self,
        user: T,
        merge_state: MergeState,
    ) -> Result<(IndexMap<PullRequest, String>, Id), GitHubError> {
        let user = user.as_ref();

        let mut pr_branch_map = IndexMap::new();
        let mut cursor = None;

        loop {
            let GraphQlResponse { data, errors } = self
                .0
                .post(GRAPHQL_URL)
                .run_graphql(GetBranches::build(GetBranchesVariables {
                    owner: user,
                    name: WINGET_PKGS,
                    cursor: cursor.as_deref(),
                }))
                .await?;

            let repository = data.and_then(|data| data.repository).ok_or_else(|| {
                GitHubError::graphql_errors(eyre!("failed to get fork"), errors.clone())
            })?;

            let default_branch = repository.default_branch_ref.ok_or_else(|| {
                GitHubError::graphql_errors(
                    eyre!("failed to get fork's default branch"),
                    errors.clone(),
                )
            })?;

            let RefConnection {
                branches,
                page_info,
            } = repository.refs.ok_or_else(|| {
                GitHubError::graphql_errors(eyre!("failed to get fork's branches"), errors)
            })?;

            for branch in branches.into_iter().filter(|branch| {
                branch.name != default_branch.name
                    && branch
                        .associated_pull_requests
                        .pull_requests
                        .iter()
                        .all(|pull_request| !pull_request.state.is_open())
            }) {
                if let Some(pull_request) = branch
                    .associated_pull_requests
                    .pull_requests
                    .into_iter()
                    .filter(|pull_request| match merge_state {
                        MergeState::MERGED => pull_request.state.is_merged(),
                        MergeState::CLOSED => pull_request.state.is_closed(),
                        _ => !pull_request.state.is_open(),
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
            input: CreatePullRequestInput::builder()
                .base_ref_name(branch_name)
                .body(body)
                .head_ref_name(fork_ref_name)
                .head_repository_id(fork_id)
                .repository_id(repository_id)
                .title(title)
                .build(),
        });

        let GraphQlResponse { data, errors } =
            self.0.post(GRAPHQL_URL).run_graphql(operation).await?;

        data.and_then(|data| data.create_pull_request?.pull_request)
            .map(|pull_request| pull_request.url)
            .ok_or_else(|| {
                GitHubError::graphql_errors(eyre!("failed to create pull request"), errors)
            })
    }

    pub async fn delete_branches<I, T>(
        &self,
        repository_id: &Id,
        branch_names: I,
    ) -> Result<(), GitHubError>
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        let GraphQlResponse { data, errors } = self
            .0
            .post(GRAPHQL_URL)
            .run_graphql(UpdateRefs::build(UpdateRefsInput::new(
                RefUpdate::delete_branches(branch_names),
                repository_id,
            )))
            .await?;

        if data.is_some() {
            Ok(())
        } else {
            Err(GitHubError::graphql_errors(
                eyre!("failed to delete branches"),
                errors,
            ))
        }
    }

    pub fn get_all_values_from_url(
        &self,
        url: DecodedUrl,
    ) -> OptionFuture<impl Future<Output = Result<GitHubValues, GitHubError>> + Sized> {
        url.path_segments()
            .and_then(|mut parts| {
                let _file_name = parts.next_back()?;
                let builder = self
                    .get_all_values()
                    .owner(parts.next()?.to_owned())
                    .repo(parts.next()?.to_owned());
                let _releases = parts.next()?;
                let _download = parts.next()?;
                Some(builder.tag_name(parts.join("/")).send())
            })
            .into()
    }

    #[builder(finish_fn = send)]
    pub async fn get_all_values<'a>(
        &self,
        #[builder(into)] owner: Cow<'a, str>,
        #[builder(into)] repo: Cow<'a, str>,
        #[builder(into)] tag_name: Cow<'a, str>,
    ) -> Result<GitHubValues, GitHubError> {
        let GraphQlResponse { data, errors } = self
            .0
            .post(GRAPHQL_URL)
            .run_graphql(GetAllValues::build(GetAllValuesVariables {
                name: &repo,
                owner: &owner,
                tag_name: &tag_name,
            }))
            .await?;

        let repository = data.and_then(|data| data.repository).ok_or_else(|| {
            GitHubError::graphql_errors(
                eyre!("failed to get repository for {owner}/{repo}"),
                errors.clone(),
            )
        })?;

        let object = repository.object.ok_or_else(|| {
            GitHubError::graphql_errors(
                eyre!("failed to get repository Git object from {owner}/{repo}"),
                errors,
            )
        })?;

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
                    format!("https://github.com/{owner}/{repo}/blob/HEAD/{name}")
                        .parse::<LicenseUrl>()
                        .ok()
                }),
            GetAllValuesGitObject::Unknown => None,
        };

        let release = repository.release;

        let topics = repository
            .topics
            .nodes
            .into_iter()
            .flat_map(|topic_node| Tag::new(topic_node.topic.name))
            .collect::<BTreeSet<_>>();

        let publisher_support_url = if repository.has_issues_enabled {
            format!("https://github.com/{owner}/{repo}/issues")
                .parse::<PublisherSupportUrl>()
                .ok()
        } else {
            None
        };

        Ok(GitHubValues {
            publisher_url: PublisherUrl::from_str(repository.owner.url.as_str())?,
            publisher_support_url,
            license: repository
                .license_info
                .and_then(|license| {
                    (!license.is_pseudo).then(|| license.spdx_id.unwrap_or(license.key))
                })
                .and_then(|license| License::new(license).ok()),
            license_url,
            package_url: repository.url.as_str().parse::<PackageUrl>()?,
            release_notes: release
                .as_ref()
                .and_then(|release| ReleaseNotes::from_html(release.description_html.as_ref()?)),
            release_notes_url: release
                .and_then(|release| release.url.as_str().parse::<ReleaseNotesUrl>().ok()),
            topics,
        })
    }

    #[builder(finish_fn = send)]
    pub async fn remove_version(
        &self,
        identifier: &PackageIdentifier,
        version: &PackageVersion,
        reason: &str,
        fork: &RepositoryData,
        winget_pkgs: &RepositoryData,
        #[builder(default)] issue_resolves: &[NonZeroU32],
    ) -> Result<Url, GitHubError> {
        // Create an indeterminate progress bar to show as a pull request is being created
        let pr_progress = ProgressBar::new_spinner().with_message(format!(
            "Creating a pull request to remove {identifier} {version}",
        ));
        pr_progress.enable_steady_tick(SPINNER_TICK_RATE);

        let branch_name = branch_name(identifier, version);
        let pull_request_branch = self
            .create_branch(
                &fork.id,
                &branch_name,
                winget_pkgs.default_branch_oid.clone(),
            )
            .await?;
        let commit_title = CommitTitle::remove(identifier, version).to_string();
        let deletions = self
            .get_directory_content()
            .owner(&fork.owner)
            .branch_name(&branch_name)
            .path(&PackagePath::new(identifier, Some(version), None))
            .call()
            .await?
            .map(FileDeletion::new)
            .collect::<Vec<_>>();
        let _commit_url = self
            .commit()
            .branch_id(&pull_request_branch.id)
            .head_sha(pull_request_branch.target.unwrap())
            .message(&commit_title)
            .deletions(deletions)
            .create()
            .await?;
        let pull_request_url = self
            .create_pull_request(
                &winget_pkgs.id,
                &fork.id,
                &format!("{}:{}", fork.owner, pull_request_branch.name),
                &winget_pkgs.default_branch_name,
                &commit_title,
                &pull_request_body()
                    .issue_resolves(issue_resolves)
                    .alternative_text(reason)
                    .build(),
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
        issue_resolves: &[NonZeroU32],
        created_with: Option<&str>,
        created_with_url: Option<&DecodedUrl>,
    ) -> Result<Url, GitHubError> {
        let (current_user, winget_pkgs) =
            tokio::try_join!(self.get_username(), self.get_winget_pkgs().send())?;
        let fork = self.get_winget_pkgs().owner(&current_user).send().await?;
        let branch_name = branch_name(identifier, version);
        let pull_request_branch = self
            .create_branch(&fork.id, &branch_name, winget_pkgs.default_branch_oid)
            .await?;
        let commit_title = commit_title(identifier, version, UpdateState::get(version, versions));
        let additions = changes
            .iter()
            .map(|(path, content)| FileAddition::new(path, content))
            .collect::<Vec<_>>();
        let deletions = if replace_version.is_some() {
            self.get_directory_content()
                .owner(&current_user)
                .branch_name(&branch_name)
                .path(&PackagePath::new(identifier, replace_version, None))
                .call()
                .await?
                .map(FileDeletion::new)
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        let _commit_url = self
            .commit()
            .branch_id(&pull_request_branch.id)
            .head_sha(pull_request_branch.target.unwrap())
            .message(&commit_title)
            .additions(additions)
            .deletions(deletions)
            .create()
            .await?;
        self.create_pull_request(
            &winget_pkgs.id,
            &fork.id,
            &format!("{current_user}:{}", pull_request_branch.name),
            &winget_pkgs.default_branch_name,
            &commit_title,
            &pull_request_body()
                .issue_resolves(issue_resolves)
                .maybe_created_with(created_with)
                .maybe_created_with_url(created_with_url)
                .build(),
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
    pub topics: BTreeSet<Tag>,
}

pub struct GitHubFile {
    pub name: String,
    pub text: String,
}

impl GitHubFile {
    pub fn new<T, S>(name: T, text: S) -> Self
    where
        T: Into<String>,
        S: Into<String>,
    {
        Self {
            name: name.into(),
            text: text.into(),
        }
    }
}

pub struct RepositoryData {
    pub id: Id,
    pub owner: String,
    pub full_name: String,
    pub url: Url,
    pub default_branch_name: String,
    pub default_branch_oid: GitObjectId,
    pub default_branch_ref_id: Id,
    pub commit_count: i32,
}
