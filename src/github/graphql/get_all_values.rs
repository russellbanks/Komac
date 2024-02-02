use crate::github::graphql::github_schema::github_schema as schema;
use url::Url;

/*
query GetAllValues($owner: String!, $name: String!, $tagName: String!) {
  repository(owner: $owner, name: $name) {
    description
    hasIssuesEnabled
    isInOrganization
    licenseInfo {
      key
      pseudoLicense
      spdxId
    }
    object(expression: "HEAD:") {
      ... on Tree {
        entries {
          name
          type
        }
      }
    }
    release(tagName: $tagName) {
      description
      url
    }
    repositoryTopics(first: 16) {
      nodes {
        topic {
          name
        }
      }
    }
    url
  }
  organization(login: $owner) {
    url
    websiteUrl
  }
  user(login: $owner) {
    url
  }
}
*/

#[derive(cynic::QueryVariables)]
pub struct GetAllValuesVariables<'a> {
    pub name: &'a str,
    pub owner: &'a str,
    pub tag_name: &'a str,
}

#[derive(cynic::QueryFragment)]
pub struct Tree {
    #[cynic(flatten)]
    pub entries: Vec<TreeEntry>,
}

#[derive(cynic::QueryFragment)]
pub struct TreeEntry {
    pub name: String,
    #[cynic(rename = "type")]
    pub type_: String,
}

#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Query", variables = "GetAllValuesVariables")]
pub struct GetAllValues {
    #[arguments(owner: $owner, name: $name)]
    pub repository: Option<Repository>,
    #[arguments(login: $owner)]
    pub organization: Option<Organization>,
    #[arguments(login: $owner)]
    pub user: Option<User>,
}

#[derive(cynic::QueryFragment)]
pub struct User {
    pub url: Url,
}

#[derive(cynic::QueryFragment)]
#[cynic(variables = "GetAllValuesVariables")]
pub struct Repository {
    pub description: Option<String>,
    pub has_issues_enabled: bool,
    pub is_in_organization: bool,
    pub license_info: Option<License>,
    #[arguments(expression: "HEAD:")]
    pub object: Option<GetAllValuesGitObject>,
    #[arguments(tagName: $tag_name)]
    pub release: Option<Release>,
    #[arguments(first: 16)]
    pub repository_topics: RepositoryTopicConnection,
    pub url: Url,
}

#[derive(cynic::QueryFragment)]
pub struct RepositoryTopicConnection {
    #[cynic(flatten)]
    pub nodes: Vec<RepositoryTopic>,
}

#[derive(cynic::QueryFragment)]
pub struct RepositoryTopic {
    pub topic: Topic,
}

#[derive(cynic::QueryFragment)]
pub struct Topic {
    pub name: String,
}

#[derive(cynic::QueryFragment)]
pub struct Release {
    pub description: Option<String>,
    pub url: Url,
}

#[derive(cynic::QueryFragment)]
pub struct Organization {
    pub url: Url,
    pub website_url: Option<Url>,
}

#[derive(cynic::QueryFragment)]
pub struct License {
    pub key: String,
    pub pseudo_license: bool,
    pub spdx_id: Option<String>,
}

#[derive(cynic::InlineFragments)]
#[cynic(graphql_type = "GitObject")]
pub enum GetAllValuesGitObject {
    Tree(Tree),
    #[cynic(fallback)]
    Unknown,
}
