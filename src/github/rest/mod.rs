use reqwest::header::HeaderValue;

pub mod get_tree;

pub const GITHUB_JSON_MIME: HeaderValue = HeaderValue::from_static("application/vnd.github+json");
