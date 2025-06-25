use serde::Deserialize;

use crate::installers::burn::manifest::yes_no::YesNoButton;

#[expect(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Provides<'manifest> {
    #[serde(rename = "@Key")]
    pub key: &'manifest str,
    #[serde(rename = "@Version")]
    pub version: Option<&'manifest str>,
    #[serde(rename = "@DisplayName")]
    pub display_name: Option<&'manifest str>,
    #[serde(rename = "@Imported", default)]
    pub imported: YesNoButton,
}
