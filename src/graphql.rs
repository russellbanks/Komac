use crate::graphql::create_commit::{
    CommitMessage, CommittableBranch, CreateCommitOnBranchInput, FileChanges,
};
use crate::graphql::create_pull_request::CreatePullRequestInput;
use crate::graphql::get_all_values::GetAllValuesRepositoryObject;
use crate::graphql::get_deep_directory_content::{
    GetDeepDirectoryContentRepositoryObject,
    GetDeepDirectoryContentRepositoryObjectOnTreeEntriesObject,
};
use crate::graphql::get_directory_content::GetDirectoryContentRepositoryObject;
use crate::graphql::get_directory_content_with_text::{
    GetDirectoryContentWithTextRepositoryObject,
    GetDirectoryContentWithTextRepositoryObjectOnTreeEntriesObject,
};
use crate::types::license::License;
use crate::types::package_version::PackageVersion;
use crate::types::release_notes::ReleaseNotes;
use crate::types::urls::license_url::LicenseUrl;
use crate::types::urls::package_url::PackageUrl;
use crate::types::urls::publisher_url::PublisherUrl;
use crate::types::urls::release_notes_url::ReleaseNotesUrl;
use color_eyre::eyre::{bail, eyre, Result};
use graphql_client::reqwest::post_graphql;
use graphql_client::GraphQLQuery;
use reqwest::Client;
use std::fmt::{Display, Formatter};
use std::ops::Not;
use std::str::FromStr;
use url::Url;

type GitObjectID = String;
type Base64String = String;
type URI = Url;

const GITHUB_GRAPHQL_URL: &str = "https://api.github.com/graphql";

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.graphql",
    query_path = "src/graphql/queries.graphql"
)]
struct GetRepositoryInfo;

pub struct RepositoryData {
    pub id: String,
    pub default_branch_name: String,
    pub default_branch_oid: String,
}

pub async fn get_repository_info(
    client: &Client,
    owner: &str,
    name: &str,
) -> Result<RepositoryData> {
    let variables = get_repository_info::Variables {
        owner: owner.to_owned(),
        name: name.to_owned(),
    };
    let repository = post_graphql::<GetRepositoryInfo, _>(client, GITHUB_GRAPHQL_URL, variables)
        .await?
        .data
        .ok_or_else(|| {
            eyre!("No data was returned when requesting repository information for {owner}/{name}")
        })?
        .repository
        .ok_or_else(|| {
            eyre!(
            "No repository was returned when requesting repository information for {owner}/{name}"
        )
        })?;

    let default_branch = repository.default_branch_ref
        .ok_or_else(|| eyre!("No default branch reference was returned when requesting repository information for {owner}/{name}"))?;

    let default_branch_oid = default_branch
        .target
        .ok_or_else(|| eyre!("No default branch object was returned when requesting repository information for {owner}/{name}"))
        .map(|target| target.oid)?;

    Ok(RepositoryData {
        id: repository.id,
        default_branch_name: default_branch.name,
        default_branch_oid,
    })
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.graphql",
    query_path = "src/graphql/queries.graphql"
)]
struct GetRepositoryId;

pub async fn get_repository_id(client: &Client, owner: &str, name: &str) -> Result<String> {
    let variables = get_repository_id::Variables {
        owner: owner.to_owned(),
        name: name.to_owned(),
    };
    post_graphql::<GetRepositoryId, _>(client, GITHUB_GRAPHQL_URL, variables)
        .await?
        .data
        .unwrap()
        .repository
        .ok_or_else(|| eyre!("Repository missing when retrieving repository ID for {owner}/{name}"))
        .map(|repository| repository.id)
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.graphql",
    query_path = "src/graphql/queries.graphql"
)]
struct CreateRef;

pub struct Ref {
    pub id: String,
    pub name: String,
    pub head_sha: String,
}

pub async fn create_branch(
    client: &Client,
    repository_id: &str,
    branch_name: &str,
    base_sha: &str,
) -> Result<Ref> {
    let variables = create_ref::Variables {
        repository_id: repository_id.to_owned(),
        name: format!("refs/heads/{branch_name}"),
        oid: base_sha.to_owned(),
    };
    let r#ref = post_graphql::<CreateRef, _>(client, GITHUB_GRAPHQL_URL, variables)
        .await?
        .data
        .ok_or_else(|| eyre!("No data was returned when attempting to create {branch_name}"))?
        .create_ref
        .ok_or_else(|| eyre!("Failed to create {branch_name}"))?
        .ref_
        .ok_or_else(|| eyre!("GitHub did not return a reference after creating {branch_name}"))?;

    Ok(Ref {
        id: r#ref.id,
        name: r#ref.name,
        head_sha: r#ref.target.unwrap().oid,
    })
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.graphql",
    query_path = "src/graphql/queries.graphql"
)]
struct GetCurrentUserLogin;

pub async fn get_current_user_login(client: &Client) -> Result<String> {
    post_graphql::<GetCurrentUserLogin, _>(
        client,
        GITHUB_GRAPHQL_URL,
        get_current_user_login::Variables,
    )
    .await?
    .data
    .ok_or_else(|| eyre!("No data was returned when retrieving the current user's login"))
    .map(|data| data.viewer.login)
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.graphql",
    query_path = "src/graphql/queries.graphql",
    skip_serializing_none
)]
pub struct CreateCommit;

pub async fn create_commit(
    client: &Client,
    branch_id: &str,
    head_sha: &str,
    message: &str,
    additions: Option<Vec<create_commit::FileAddition>>,
    deletions: Option<Vec<create_commit::FileDeletion>>,
) -> Result<Url> {
    let variables = create_commit::Variables {
        input: CreateCommitOnBranchInput {
            branch: CommittableBranch {
                branch_name: None,
                id: Some(branch_id.to_owned()),
                repository_name_with_owner: None,
            },
            client_mutation_id: None,
            expected_head_oid: head_sha.to_owned(),
            file_changes: Some(FileChanges {
                additions,
                deletions,
            }),
            message: CommitMessage {
                body: None,
                headline: message.to_owned(),
            },
        },
    };
    post_graphql::<CreateCommit, _>(client, GITHUB_GRAPHQL_URL, variables)
        .await?
        .data
        .ok_or_else(|| eyre!("No data was returned when attempting to create commit"))?
        .create_commit_on_branch
        .ok_or_else(|| eyre!("Failed to create commit"))?
        .commit
        .map(|commit| commit.url)
        .ok_or_else(|| eyre!("No commit data was returned when creating commit"))
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.graphql",
    query_path = "src/graphql/queries.graphql"
)]
struct GetDirectoryContent;

pub async fn get_directory_content(
    client: &Client,
    owner: &str,
    repo: &str,
    branch: &str,
    path: &str,
) -> Result<impl Iterator<Item = String>> {
    let expression = format!("{branch}:{path}");
    let variables = get_directory_content::Variables {
        owner: owner.to_owned(),
        name: repo.to_owned(),
        expression,
    };
    let object = post_graphql::<GetDirectoryContent, _>(client, GITHUB_GRAPHQL_URL, variables)
        .await?
        .data
        .ok_or_else(|| {
            eyre!("No data was returned when attempting to get the directory content of {path}")
        })?
        .repository
        .ok_or_else(|| {
            eyre!("Response is missing repository when getting the directory content of {path}")
        })?
        .object
        .ok_or_else(|| {
            eyre!("Response is missing object when getting the directory content of {path}")
        })?;

    if let GetDirectoryContentRepositoryObject::Tree(tree) = object {
        tree.entries
            .map(|entries| entries.into_iter().filter_map(|entry| entry.path))
            .ok_or_else(|| eyre!("No files were found for {path}"))
    } else {
        bail!("GitHub did not return the expected type of a tree when retrieving the directory content of {path}")
    }
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.graphql",
    query_path = "src/graphql/queries.graphql"
)]
struct GetDirectoryContentWithText;

pub struct GitHubFile {
    pub name: String,
    pub text: String,
}

pub async fn get_directory_content_with_text(
    client: &Client,
    owner: &str,
    repo: &str,
    path: &str,
) -> Result<Vec<GitHubFile>> {
    let expression = format!("HEAD:{path}");
    let variables = get_directory_content_with_text::Variables {
        owner: owner.to_owned(),
        name: repo.to_owned(),
        expression,
    };

    let data =
        post_graphql::<GetDirectoryContentWithText, _>(client, GITHUB_GRAPHQL_URL, variables)
            .await?
            .data
            .ok_or_else(|| {
                eyre!("No data was returned when attempting to get the directory content of {path}")
            })?;

    let repository = data.repository.ok_or_else(|| {
        eyre!("Response is missing repository when getting the directory content of {path}")
    })?;
    let object = repository.object.ok_or_else(|| {
        eyre!("Response is missing object when getting the directory content of {path}")
    })?;

    if let GetDirectoryContentWithTextRepositoryObject::Tree(tree) = object {
        if let Some(entries) = tree.entries {
            let files = entries
                .into_iter()
                .filter_map(|entry| {
                    if let Some(
                        GetDirectoryContentWithTextRepositoryObjectOnTreeEntriesObject::Blob(blob),
                    ) = entry.object
                    {
                        Some(GitHubFile {
                            name: entry.name,
                            text: blob.text.unwrap_or_default(),
                        })
                    } else {
                        None
                    }
                })
                .collect::<Vec<GitHubFile>>();

            if files.is_empty() {
                bail!("No files were found for {path}")
            }

            Ok(files)
        } else {
            bail!("No files were found for {path}")
        }
    } else {
        bail!("GitHub did not return the expected type of a tree when retrieving the directory content of {path}")
    }
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.graphql",
    query_path = "src/graphql/queries.graphql",
    skip_serializing_none
)]
struct CreatePullRequest;

pub async fn create_pull_request(
    client: &Client,
    repository_id: &str,
    fork_repository_id: &str,
    fork_ref_name: &str,
    default_branch_name: &str,
    title: &str,
    body: &str,
) -> Result<Url> {
    let variables = create_pull_request::Variables {
        input: CreatePullRequestInput {
            base_ref_name: default_branch_name.to_owned(),
            body: Some(body.to_owned()),
            client_mutation_id: None,
            draft: None,
            head_ref_name: fork_ref_name.to_owned(),
            head_repository_id: Some(fork_repository_id.to_owned()),
            maintainer_can_modify: None,
            repository_id: repository_id.to_owned(),
            title: title.to_owned(),
        },
    };

    post_graphql::<CreatePullRequest, _>(client, GITHUB_GRAPHQL_URL, variables)
        .await?
        .data
        .ok_or_else(|| {
            eyre!("No data was returned when creating pull request from {fork_ref_name}")
        })?
        .create_pull_request
        .ok_or_else(|| eyre!("Failed to create the pull request from {fork_ref_name}"))?
        .pull_request
        .ok_or_else(|| {
            eyre!(
            "No pull request data was returned when creating a pull request from {fork_ref_name}"
        )
        })
        .map(|pull_request| pull_request.url)
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.graphql",
    query_path = "src/graphql/queries.graphql"
)]
struct GetDeepDirectoryContent;

pub async fn get_all_versions(
    client: &Client,
    owner: &str,
    repo: &str,
    path: &str,
) -> Result<Vec<PackageVersion>> {
    let variables = get_deep_directory_content::Variables {
        owner: owner.to_owned(),
        name: repo.to_owned(),
        expression: format!("HEAD:{path}"),
    };

    let object = post_graphql::<GetDeepDirectoryContent, _>(client, GITHUB_GRAPHQL_URL, variables)
        .await?
        .data
        .ok_or_else(|| eyre!("No data was returned when getting directory content of {path}"))?
        .repository
        .ok_or_else(|| {
            eyre!("No repository was returned when getting directory content of {path}")
        })?
        .object
        .ok_or_else(|| eyre!("No object was returned when getting directory content of {path}"))?;
    if let GetDeepDirectoryContentRepositoryObject::Tree(tree) = object {
        if let Some(entries) = tree.entries {
            let files = entries
                .into_iter()
                .filter_map(|entry| {
                    if let Some(GetDeepDirectoryContentRepositoryObjectOnTreeEntriesObject::Tree(
                        tree,
                    )) = &entry.object
                    {
                        if let Some(sub_entries) = &tree.entries {
                            if sub_entries
                                .iter()
                                .filter(|entry| entry.type_ == "tree")
                                .count()
                                == 0
                            {
                                Some(entry)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .map(|entry| entry.name)
                .filter_map(|entry| PackageVersion::new(&entry).ok())
                .collect::<Vec<_>>();

            if files.is_empty() {
                bail!("No files were found for {path}")
            }

            Ok(files)
        } else {
            bail!("No files were found for {path}")
        }
    } else {
        bail!("GitHub did not return the expected type of a tree when retrieving the directory content of {path}")
    }
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.graphql",
    query_path = "src/graphql/queries.graphql"
)]
struct GetBranches;

pub type Branch = get_branches::GetBranchesRepositoryRefsNodes;

pub async fn get_branches(
    client: &Client,
    owner: &str,
    repo: &str,
) -> Result<(Vec<Branch>, String)> {
    let variables = get_branches::Variables {
        owner: owner.to_owned(),
        name: repo.to_owned(),
    };

    let repository = post_graphql::<GetBranches, _>(client, GITHUB_GRAPHQL_URL, variables)
        .await?
        .data
        .ok_or_else(|| eyre!("No data was returned when getting branches for {owner}/{repo}"))?
        .repository
        .ok_or_else(|| {
            eyre!("No repository was returned when getting branches for {owner}/{repo}")
        })?;

    let default_branch_name = repository
        .default_branch_ref
        .ok_or_else(|| {
            eyre!(
                "No default branch reference was returned when getting branches for {owner}/{repo}"
            )
        })?
        .name;

    let refs = repository
        .refs
        .ok_or_else(|| {
            eyre!("No references were returned when getting branches for {owner}/{repo}")
        })?
        .nodes
        .ok_or_else(|| eyre!("No nodes were returned when getting branches for {owner}/{repo}"))?;

    Ok((
        refs.into_iter()
            .flatten()
            .filter(|branch| branch.name != default_branch_name)
            .collect(),
        default_branch_name,
    ))
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.graphql",
    query_path = "src/graphql/queries.graphql"
)]
struct GetPullRequestFromBranch;

#[derive(Eq, Hash, PartialEq)]
pub struct PullRequest {
    pub title: String,
    pub state: PullRequestState,
    pub url: Url,
}

#[derive(Eq, Hash, PartialEq)]
pub enum PullRequestState {
    Open,
    Closed,
    Merged,
    Other(String),
}

impl Display for PullRequest {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.title)
    }
}

pub async fn get_pull_request_from_branch(
    client: &Client,
    owner: &str,
    repo: &str,
    base_ref_name: &str,
    head_ref_name: &str,
) -> Result<Option<PullRequest>> {
    let variables = get_pull_request_from_branch::Variables {
        owner: owner.to_owned(),
        name: repo.to_owned(),
        base_ref_name: base_ref_name.to_owned(),
        head_ref_name: head_ref_name.to_owned(),
    };

    let nodes = post_graphql::<GetPullRequestFromBranch, _>(client, GITHUB_GRAPHQL_URL, variables)
        .await?
        .data
        .ok_or_else(|| eyre!("No data was returned when getting an associated pull request for {head_ref_name} to {owner}/{repo}"))?
        .repository
        .ok_or_else(|| eyre!("No repository was returned when getting an associated pull request for {head_ref_name} to {owner}/{repo}"))?
        .pull_requests
        .nodes
        .ok_or_else(|| eyre!("No nodes were returned when getting an associated pull request for {head_ref_name} to {owner}/{repo}"))?;

    Ok(nodes.into_iter().find_map(|nodes_opt| {
        nodes_opt.map(|pr_nodes| PullRequest {
            title: pr_nodes.title,
            state: match pr_nodes.state {
                get_pull_request_from_branch::PullRequestState::OPEN => PullRequestState::Open,
                get_pull_request_from_branch::PullRequestState::CLOSED => PullRequestState::Closed,
                get_pull_request_from_branch::PullRequestState::MERGED => PullRequestState::Merged,
                get_pull_request_from_branch::PullRequestState::Other(str) => {
                    PullRequestState::Other(str)
                }
            },
            url: pr_nodes.url,
        })
    }))
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.graphql",
    query_path = "src/graphql/queries.graphql"
)]
struct GetAllValues;

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

pub async fn get_all_values(
    client: &Client,
    owner: String,
    repo: String,
    tag_name: String,
) -> Result<GitHubValues> {
    let variables = get_all_values::Variables {
        owner: owner.clone(),
        name: repo.clone(),
        tag_name: tag_name.clone(),
    };

    let data = post_graphql::<GetAllValues, _>(client, GITHUB_GRAPHQL_URL, variables)
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
        GetAllValuesRepositoryObject::Tree(tree) => {
            tree.entries
                .ok_or_else(|| eyre!("No directory entries were returned when getting root directory content for {owner}/{repo}"))?
                .into_iter()
                .filter_map(|entry| (entry.type_ == "blob").then_some(entry.name))
                .find(|name| name
                    .rfind('.')
                    .map_or(
                        name.to_ascii_lowercase(),
                        |dot_index| name[..dot_index].to_ascii_lowercase()
                    ) == "license"
                )
                .map(|name| LicenseUrl::from_str(&format!("https://github.com/{owner}/{repo}/blob/HEAD/{name}")))
        }
        _ => None,
    }.transpose()?;

    let release = repository
        .release
        .ok_or_else(|| eyre!("No release was found with the tag of {tag_name}"))?;

    let topics = repository
        .repository_topics
        .nodes
        .unwrap_or_default()
        .into_iter()
        .filter_map(|topic_node| topic_node.map(|node| node.topic.name))
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

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.graphql",
    query_path = "src/graphql/queries.graphql"
)]
struct DeleteRef;

pub async fn delete_ref(client: &Client, ref_id: &str) -> Result<()> {
    let variables = delete_ref::Variables {
        ref_: ref_id.to_string(),
    };

    let response = post_graphql::<DeleteRef, _>(client, GITHUB_GRAPHQL_URL, variables).await?;

    if response.data.is_some() {
        Ok(())
    } else {
        bail!("Failed to delete ref with id {ref_id}")
    }
}
