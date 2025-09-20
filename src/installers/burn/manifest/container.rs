use std::borrow::Cow;

use serde::Deserialize;

use super::bool_from_yes_no;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Container<'manifest> {
    #[serde(rename = "@Id")]
    pub id: &'manifest str,
    #[serde(rename = "@FileSize")]
    pub file_size: u32,
    #[serde(rename = "@Hash")]
    pub hash: &'manifest str,
    #[serde(rename = "@DownloadUrl")]
    pub download_url: Option<Cow<'manifest, str>>,
    #[serde(rename = "@FilePath")]
    pub file_path: &'manifest str,
    #[serde(rename = "@AttachedIndex")]
    pub attached_index: Option<u32>,
    #[serde(rename = "@Attached", deserialize_with = "bool_from_yes_no", default)]
    pub attached: bool,
    #[serde(rename = "@Primary", deserialize_with = "bool_from_yes_no", default)]
    pub primary: bool,
}
