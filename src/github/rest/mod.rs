use reqwest::header::HeaderValue;

pub mod get_tree;

#[expect(clippy::declare_interior_mutable_const)]
pub const GITHUB_JSON_MIME: HeaderValue = HeaderValue::from_static("application/vnd.github+json");
