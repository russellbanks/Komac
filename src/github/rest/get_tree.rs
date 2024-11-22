use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GitTree {
    pub sha: String,
    pub url: String,
    pub truncated: bool,
    pub tree: Vec<TreeObject>,
}

#[derive(Serialize, Deserialize)]
pub struct TreeObject {
    pub path: String,
    pub mode: String,
    pub r#type: String,
    pub sha: String,
    pub size: Option<i32>,
    pub url: String,
}
