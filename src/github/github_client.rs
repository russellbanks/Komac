use crate::credential::get_default_headers;
use crate::github::graphql::create_commit::{
    CommitMessage, CommittableBranch, CreateCommit, CreateCommitOnBranchInput,
    CreateCommitVariables, FileAddition, FileChanges, FileDeletion,
};
use crate::github::graphql::create_pull_request::{
    CreatePullRequest, CreatePullRequestInput, CreatePullRequestVariables,
};
use crate::github::graphql::create_ref::{CreateRef, CreateRefVariables, Ref as CreateBranchRef};
use crate::github::graphql::delete_ref::{DeleteRef, DeleteRefVariables};
use crate::github::graphql::get_all_values::{
    GetAllValues, GetAllValuesGitObject, GetAllValuesVariables,
};
use crate::github::graphql::get_branches::{GetBranches, Ref as GetBranchRef};
use crate::github::graphql::get_current_user_login::GetCurrentUserLogin;
use crate::github::graphql::get_deep_directory_content::{
    DeepGitObject, GetDeepDirectoryContent, GitObject2,
};
use crate::github::graphql::get_directory_content::{
    GetDirectoryContent, GetDirectoryContentVariables, TreeGitObject,
};
use crate::github::graphql::get_directory_content_with_text::{
    DirTextTreeGitObject, GetDirectoryContentWithText, GitObject,
};
use crate::github::graphql::get_pull_request_from_branch::{
    GetPullRequestFromBranch, GetPullRequestFromBranchVariables, PullRequest,
};
use crate::github::graphql::get_repository_info::{
    GetRepositoryInfo, GitObjectId, RepositoryVariables,
};
use crate::github::utils::get_package_path;
use crate::manifests::default_locale_manifest::DefaultLocaleManifest;
use crate::manifests::installer_manifest::InstallerManifest;
use crate::manifests::locale_manifest::LocaleManifest;
use crate::manifests::version_manifest::VersionManifest;
use crate::types::license::License;
use crate::types::package_identifier::PackageIdentifier;
use crate::types::package_version::PackageVersion;
use crate::types::release_notes::ReleaseNotes;
use crate::types::urls::license_url::LicenseUrl;
use crate::types::urls::package_url::PackageUrl;
use crate::types::urls::publisher_url::PublisherUrl;
use crate::types::urls::release_notes_url::ReleaseNotesUrl;
use color_eyre::eyre::{bail, eyre, OptionExt, Result};
use const_format::formatcp;
use cynic::http::ReqwestExt;
use cynic::{Id, MutationBuilder, QueryBuilder};
use reqwest::Client;
use std::env;
use std::ops::Not;
use std::path::Path;
use std::str::FromStr;
use url::Url;

pub const MICROSOFT: &str = "Microsoft";
pub const WINGET_PKGS: &str = "winget-pkgs";
pub const WINGET_PKGS_FULL_NAME: &str = formatcp!("{MICROSOFT}/{WINGET_PKGS}");
const GITHUB_GRAPHQL_URL: &str = "https://api.github.com/graphql";

pub struct GitHub(Client);

impl GitHub {
    pub fn new(token: String) -> Result<Self> {
        Ok(Self(
            Client::builder()
                .default_headers(get_default_headers(Some(&token)))
                .build()?,
        ))
    }

    pub async fn get_versions(&self, path: &str) -> Result<Vec<PackageVersion>> {
        Self::get_all_versions(&self.0, MICROSOFT, WINGET_PKGS, path).await
    }

    async fn get_all_versions(
        client: &Client,
        owner: &str,
        repo: &str,
        path: &str,
    ) -> Result<Vec<PackageVersion>> {
        let files = client
            .post(GITHUB_GRAPHQL_URL)
            .run_graphql(GetDeepDirectoryContent::build(
                GetDirectoryContentVariables {
                    expression: &format!("HEAD:{path}"),
                    name: repo,
                    owner,
                },
            ))
            .await?
            .data
            .and_then(|data| data.repository)
            .and_then(|repository| repository.object)
            .and_then(|object| {
                if let GitObject2::Tree2(tree) = object {
                    return Some(tree.entries);
                }
                None
            })
            .ok_or_else(|| eyre!("Failed to retrieve directory content of {path}"))?
            .into_iter()
            .filter_map(|entry| {
                if let Some(DeepGitObject::Tree(tree)) = &entry.object {
                    if tree.entries.iter().all(|entry| entry.type_ != "tree") {
                        return Some(entry.name);
                    }
                }
                None
            })
            .filter_map(|entry| PackageVersion::new(&entry).ok())
            .collect::<Vec<_>>();

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
            .find(|file| file.name == format!("{identifier}.yaml"))
            .map(|file| serde_yaml::from_str::<VersionManifest>(&file.text))
            .transpose()?
            .ok_or_else(|| eyre!("No version manifest was found in {full_package_path}"))?;

        let locale_manifests = content
            .iter()
            .filter(|file| {
                file.name.starts_with(&format!("{identifier}.locale."))
                    && !file.name.contains(version_manifest.default_locale.as_str())
                    && Path::new(&file.name)
                        .extension()
                        .map_or(false, |ext| ext.eq_ignore_ascii_case("yaml"))
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

    async fn get_directory_content_with_text(
        client: &Client,
        owner: &str,
        repo: &str,
        path: &str,
    ) -> Result<impl Iterator<Item = GitHubFile>> {
        client
            .post(GITHUB_GRAPHQL_URL)
            .run_graphql(GetDirectoryContentWithText::build(
                GetDirectoryContentVariables {
                    expression: &format!("HEAD:{path}"),
                    name: repo,
                    owner,
                },
            ))
            .await?
            .data
            .and_then(|data| data.repository)
            .and_then(|repository| repository.object)
            .and_then(|object| {
                if let DirTextTreeGitObject::Tree(tree) = object {
                    Some(tree.entries)
                } else {
                    None
                }
            })
            .map(|entries| {
                entries.into_iter().filter_map(|entry| {
                    if let Some(GitObject::Blob(blob)) = entry.object {
                        Some(GitHubFile {
                            name: entry.name,
                            text: blob.text.unwrap_or_default(),
                        })
                    } else {
                        None
                    }
                })
            })
            .ok_or_else(|| {
                eyre!(
                "No directory content was returned when retrieving the directory content of {path}"
            )
            })
    }

    pub async fn get_username(&self) -> Result<String> {
        const KOMAC_FORK_OWNER: &str = "KOMAC_FORK_OWNER";
        match env::var(KOMAC_FORK_OWNER) {
            Ok(login) => Ok(login),
            _ => self
                .0
                .post(GITHUB_GRAPHQL_URL)
                .run_graphql(GetCurrentUserLogin::build(()))
                .await?
                .data
                .map(|data| data.viewer.login)
                .ok_or_eyre("No data was returned when retrieving the current user's login"),
        }
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
            .run_graphql(GetRepositoryInfo::build(RepositoryVariables {
                name,
                owner,
            }))
            .await?
            .data
            .and_then(|data| data.repository)
            .ok_or_else(|| {
                eyre!(
            "No repository was returned when requesting repository information for {owner}/{name}"
        )
            })?;

        let default_branch = repository.default_branch_ref
            .ok_or_else(|| eyre!("No default branch reference was returned when requesting repository information for {owner}/{name}"))?;

        let default_branch_oid = default_branch
            .target
            .map(|target| target.oid)
            .ok_or_else(|| eyre!("No default branch object was returned when requesting repository information for {owner}/{name}"))?;

        Ok(RepositoryData {
            id: repository.id,
            default_branch_name: default_branch.name,
            default_branch_oid,
        })
    }

    pub async fn create_branch(
        &self,
        fork_id: &Id,
        branch_name: &str,
        oid: &str,
    ) -> Result<CreateBranchRef> {
        let r#ref = self
            .0
            .post(GITHUB_GRAPHQL_URL)
            .run_graphql(CreateRef::build(CreateRefVariables {
                name: &format!("refs/heads/{branch_name}"),
                oid: GitObjectId(oid.to_owned()),
                repository_id: fork_id,
            }))
            .await?
            .data
            .and_then(|data| data.create_ref)
            .and_then(|create_ref| create_ref.ref_)
            .ok_or_else(|| eyre!("No reference was returned after creating {branch_name}"))?;

        Ok(r#ref)
    }

    pub async fn create_commit(
        &self,
        branch_id: &Id,
        head_sha: &str,
        message: &str,
        additions: Option<Vec<FileAddition<'_>>>,
        deletions: Option<Vec<FileDeletion<'_>>>,
    ) -> Result<Url> {
        self.0
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
            .await?
            .data
            .and_then(|data| data.create_commit_on_branch)
            .and_then(|commit_object| commit_object.commit)
            .map(|commit| commit.url)
            .ok_or_eyre("No commit data was returned when creating commit")
    }

    pub async fn get_directory_content(
        &self,
        owner: &str,
        branch_name: &str,
        path: &str,
    ) -> Result<impl Iterator<Item = String> + Sized> {
        let entries = self
            .0
            .post(GITHUB_GRAPHQL_URL)
            .run_graphql(GetDirectoryContent::build(GetDirectoryContentVariables {
                expression: &format!("{branch_name}:{path}"),
                name: WINGET_PKGS,
                owner,
            }))
            .await?
            .data
            .and_then(|data| data.repository)
            .and_then(|repository| repository.object)
            .and_then(|object| {
                if let TreeGitObject::Tree(tree) = object {
                    return Some(tree.entries);
                }
                None
            })
            .ok_or_else(|| eyre!("No directory content was returned for {path}"))?;

        Ok(entries.into_iter().filter_map(|entry| entry.path))
    }

    pub async fn get_pull_request_from_branch(
        &self,
        default_branch_name: &str,
        branch_name: &str,
    ) -> Result<Option<PullRequest>> {
        let operation = GetPullRequestFromBranch::build(GetPullRequestFromBranchVariables {
            base_ref_name: default_branch_name,
            head_ref_name: branch_name,
            name: WINGET_PKGS,
            owner: MICROSOFT,
        });
        let mut nodes = self.0.post(GITHUB_GRAPHQL_URL).run_graphql(operation).await?
            .data
            .and_then(|data| data.repository)
            .map(|repository| repository.pull_requests.nodes)
            .ok_or_else(|| eyre!("No data was returned when getting an associated pull request for {branch_name} to {MICROSOFT}/{WINGET_PKGS}"))?;

        if nodes.is_empty() {
            Ok(None)
        } else {
            Ok(Some(nodes.swap_remove(0)))
        }
    }

    pub async fn get_branches(&self, user: &str) -> Result<(Vec<GetBranchRef>, String)> {
        let repository = self
            .0
            .post(GITHUB_GRAPHQL_URL)
            .run_graphql(GetBranches::build(RepositoryVariables {
                name: WINGET_PKGS,
                owner: user,
            }))
            .await?
            .data
            .and_then(|data| data.repository)
            .ok_or_else(|| {
                eyre!("No repository was returned when getting branches for {user}/{WINGET_PKGS}")
            })?;

        let default_branch_name = repository
            .default_branch_ref
            .map(|default_branch_ref| default_branch_ref.name)
            .ok_or_else(|| {
                eyre!(
                "No default branch reference was returned when getting branches for {user}/{WINGET_PKGS}"
            )
            })?;

        let refs = repository.refs.map(|refs| refs.nodes).ok_or_else(|| {
            eyre!("No references were returned when getting branches for {user}/{WINGET_PKGS}")
        })?;

        let branches = refs
            .into_iter()
            .filter(|branch| branch.name != default_branch_name)
            .collect();

        Ok((branches, default_branch_name))
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
        self.0
            .post(GITHUB_GRAPHQL_URL)
            .run_graphql(operation)
            .await?
            .data
            .and_then(|data| data.create_pull_request)
            .and_then(|create_pull_request| create_pull_request.pull_request)
            .map(|pull_request| pull_request.url)
            .ok_or_else(|| {
                eyre!(
            "No pull request data was returned when creating a pull request from {fork_ref_name}"
        )
            })
    }

    pub async fn delete_branch(&self, branch_id: &Id) -> Result<()> {
        let response = self
            .0
            .post(GITHUB_GRAPHQL_URL)
            .run_graphql(DeleteRef::build(DeleteRefVariables { ref_: branch_id }))
            .await?;
        if response.data.is_some() {
            Ok(())
        } else {
            bail!("Failed to delete ref with id {:?}", branch_id)
        }
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
            eyre!(
        "No directory content was returned when getting root directory content for {owner}/{repo}"
    )
        })?;

        let license_url = match object {
            GetAllValuesGitObject::Tree(tree) => tree
                .entries
                .into_iter()
                .filter_map(|entry| (entry.type_ == "blob").then_some(entry.name))
                .find(|name| {
                    name.rfind('.')
                        .map_or(name.to_ascii_lowercase(), |dot_index| {
                            name[..dot_index].to_ascii_lowercase()
                        })
                        == "license"
                })
                .map(|name| {
                    LicenseUrl::from_str(&format!(
                        "https://github.com/{owner}/{repo}/blob/HEAD/{name}"
                    ))
                }),
            GetAllValuesGitObject::Unknown => None,
        }
        .transpose()?;

        let release = repository
            .release
            .ok_or_else(|| eyre!("No release was found with the tag of {tag_name}"))?;

        let topics = repository
            .repository_topics
            .nodes
            .into_iter()
            .map(|topic_node| topic_node.topic.name)
            .collect::<Vec<_>>();

        let publisher_url = if repository.is_in_organization {
            data.organization
                .map(|org| org.website_url.unwrap_or(org.url))
                .unwrap()
        } else {
            data.user.map(|user| user.url).unwrap()
        };

        let publisher_support_url = repository
            .has_issues_enabled
            .then(|| format!("https://github.com/{owner}/{repo}/issues"));

        Ok(GitHubValues {
            publisher_url: PublisherUrl::from_str(publisher_url.as_str())?,
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
                .and_then(|license| License::new(license).ok()),
            license_url,
            package_url: PackageUrl::from_str(repository.url.as_str())?,
            release_notes: release
                .description
                .and_then(|body| ReleaseNotes::format(&body, &owner, &repo)),
            release_notes_url: ReleaseNotesUrl::from_str(release.url.as_str())?,
            has_issues_enabled: repository.has_issues_enabled,
            topics,
        })
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
    pub publisher_support_url: Option<String>,
    pub short_description: String,
    pub license: Option<License>,
    pub license_url: Option<LicenseUrl>,
    pub package_url: PackageUrl,
    pub release_notes: Option<ReleaseNotes>,
    pub release_notes_url: ReleaseNotesUrl,
    pub has_issues_enabled: bool,
    pub topics: Vec<String>,
}

pub struct GitHubFile {
    pub name: String,
    pub text: String,
}

pub struct RepositoryData {
    pub id: Id,
    pub default_branch_name: String,
    pub default_branch_oid: GitObjectId,
}
