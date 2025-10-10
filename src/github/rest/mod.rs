mod tree;

use reqwest::header::{HeaderName, HeaderValue};

pub const GITHUB_JSON_MIME: HeaderValue = HeaderValue::from_static("application/vnd.github+json");

pub const X_GITHUB_API_VERSION: HeaderName = HeaderName::from_static("x-github-api-version");

pub const REST_API_URL: &str = "https://api.github.com";

pub const REST_API_VERSION: HeaderValue = HeaderValue::from_static("2022-11-28");
